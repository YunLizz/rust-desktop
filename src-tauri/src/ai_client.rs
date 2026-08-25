//! AI 客户端：OpenAI 兼容协议 + Anthropic 协议，SSE 流式输出
//! 在工作线程中运行，通过 channel 向 UI 推送增量事件

use std::io::BufRead;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;

use crate::store::AiSettings;

pub enum AiEvent {
    Chunk(String),
    Done,
    Error(String),
}

pub fn stream_chat(
    cfg: &AiSettings,
    messages: &[(String, String)],
    tx: Sender<AiEvent>,
    cancel: Arc<AtomicBool>,
) {
    let cfg = cfg.clone();
    let messages: Vec<(String, String)> = messages.to_vec();
    std::thread::spawn(move || {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(std::time::Duration::from_secs(15))
            .timeout_read(std::time::Duration::from_secs(cfg.timeout_secs))
            .build();
        let result = if cfg.protocol == "anthropic" {
            stream_anthropic(&agent, &cfg, &messages, &tx, &cancel)
        } else {
            stream_openai(&agent, &cfg, &messages, &tx, &cancel)
        };
        match result {
            Ok(()) => {
                let _ = tx.send(AiEvent::Done);
            }
            Err(e) => {
                if cancel.load(Ordering::Relaxed) {
                    let _ = tx.send(AiEvent::Error("已取消".into()));
                } else {
                    let _ = tx.send(AiEvent::Error(e));
                }
            }
        }
    });
}

fn stream_openai(
    agent: &ureq::Agent,
    cfg: &AiSettings,
    messages: &[(String, String)],
    tx: &Sender<AiEvent>,
    cancel: &Arc<AtomicBool>,
) -> Result<(), String> {
    let base = cfg.base_url.trim_end_matches('/');
    let url = format!("{}/chat/completions", base);
    let mut msgs = vec![serde_json::json!({
        "role": "system",
        "content": cfg.system_prompt
    })];
    for (role, content) in messages {
        msgs.push(serde_json::json!({"role": role, "content": content}));
    }
    let body = serde_json::json!({
        "model": cfg.model,
        "messages": msgs,
        "stream": true,
        "temperature": cfg.temperature,
        "max_tokens": cfg.max_tokens,
    });
    let resp = agent
        .post(&url)
        .set("Content-Type", "application/json")
        .set("Authorization", &format!("Bearer {}", cfg.api_key))
        .send_json(body)
        .map_err(|e| format!("请求失败: {}", describe_ureq_error(&e)))?;
    let mut reader = std::io::BufReader::new(resp.into_reader());
    let mut line = String::new();
    loop {
        if cancel.load(Ordering::Relaxed) {
            return Err("已取消".into());
        }
        line.clear();
        let n = reader
            .read_line(&mut line)
            .map_err(|e| format!("读取响应流失败: {}", e))?;
        if n == 0 {
            break;
        }
        let l = line.trim();
        if let Some(data) = l.strip_prefix("data:") {
            let data = data.trim();
            if data == "[DONE]" {
                break;
            }
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(data) {
                if let Some(delta) = v["choices"][0]["delta"]["content"].as_str() {
                    if !delta.is_empty() {
                        let _ = tx.send(AiEvent::Chunk(delta.to_string()));
                    }
                }
            }
        }
    }
    Ok(())
}

fn stream_anthropic(
    agent: &ureq::Agent,
    cfg: &AiSettings,
    messages: &[(String, String)],
    tx: &Sender<AiEvent>,
    cancel: &Arc<AtomicBool>,
) -> Result<(), String> {
    let base = cfg.base_url.trim_end_matches('/');
    let url = if base.ends_with("/v1") {
        format!("{}/messages", base)
    } else {
        format!("{}/v1/messages", base)
    };
    let msgs: Vec<serde_json::Value> = messages
        .iter()
        .map(|(role, content)| serde_json::json!({"role": role, "content": content}))
        .collect();
    let body = serde_json::json!({
        "model": cfg.model,
        "max_tokens": cfg.max_tokens,
        "system": cfg.system_prompt,
        "messages": msgs,
        "stream": true,
        "temperature": cfg.temperature,
    });
    let resp = agent
        .post(&url)
        .set("Content-Type", "application/json")
        .set("x-api-key", &cfg.api_key)
        .set("anthropic-version", "2023-06-01")
        .send_json(body)
        .map_err(|e| format!("请求失败: {}", describe_ureq_error(&e)))?;
    let mut reader = std::io::BufReader::new(resp.into_reader());
    let mut line = String::new();
    loop {
        if cancel.load(Ordering::Relaxed) {
            return Err("已取消".into());
        }
        line.clear();
        let n = reader
            .read_line(&mut line)
            .map_err(|e| format!("读取响应流失败: {}", e))?;
        if n == 0 {
            break;
        }
        let l = line.trim();
        if let Some(data) = l.strip_prefix("data:") {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(data.trim()) {
                if v["type"] == "content_block_delta" {
                    if let Some(text) = v["delta"]["text"].as_str() {
                        if !text.is_empty() {
                            let _ = tx.send(AiEvent::Chunk(text.to_string()));
                        }
                    }
                } else if v["type"] == "error" {
                    let msg = v["error"]["message"].as_str().unwrap_or("未知错误");
                    return Err(msg.to_string());
                }
            }
        }
    }
    Ok(())
}

fn describe_ureq_error(e: &ureq::Error) -> String {
    match e {
        ureq::Error::Status(code, resp) => {
            let reason = resp.status_text().to_string();
            format!("HTTP {} {}", code, reason)
        }
        ureq::Error::Transport(t) => t.to_string(),
    }
}

/// 测试连接：非流式发送一条极短消息
pub fn test_connection(cfg: &AiSettings) -> Result<String, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(10))
        .timeout_read(std::time::Duration::from_secs(60))
        .build();
    if cfg.protocol == "anthropic" {
        let base = cfg.base_url.trim_end_matches('/');
        let url = if base.ends_with("/v1") {
            format!("{}/messages", base)
        } else {
            format!("{}/v1/messages", base)
        };
        let body = serde_json::json!({
            "model": cfg.model,
            "max_tokens": 32,
            "messages": [{"role":"user","content":"你好"}],
            "stream": false,
        });
        let resp = agent
            .post(&url)
            .set("Content-Type", "application/json")
            .set("x-api-key", &cfg.api_key)
            .set("anthropic-version", "2023-06-01")
            .send_json(body)
            .map_err(|e| describe_ureq_error(&e))?;
        let v: serde_json::Value = resp.into_json().map_err(|e| e.to_string())?;
        if let Some(t) = v["content"][0]["text"].as_str() {
            Ok(t.chars().take(50).collect())
        } else {
            Err(v.to_string().chars().take(200).collect())
        }
    } else {
        let base = cfg.base_url.trim_end_matches('/');
        let url = format!("{}/chat/completions", base);
        let body = serde_json::json!({
            "model": cfg.model,
            "messages": [{"role":"user","content":"你好"}],
            "stream": false,
            "max_tokens": 32,
        });
        let resp = agent
            .post(&url)
            .set("Content-Type", "application/json")
            .set("Authorization", &format!("Bearer {}", cfg.api_key))
            .send_json(body)
            .map_err(|e| describe_ureq_error(&e))?;
        let v: serde_json::Value = resp.into_json().map_err(|e| e.to_string())?;
        if let Some(t) = v["choices"][0]["message"]["content"].as_str() {
            Ok(t.chars().take(50).collect())
        } else {
            Err(v.to_string().chars().take(200).collect())
        }
    }
}
