//! 导出：txt / md / jsb（密码加密备份），以及 jsb 导入

use crate::model::Novel;
use crate::store::{crypto, Store};
use std::path::Path;

/// 导出纯文本
pub fn export_txt(novel: &Novel, chapters: &[(String, String, String)], path: &Path) -> Result<(), String> {
    let mut out = String::new();
    out.push_str(&format!("《{}》\n", novel.meta.title));
    if !novel.meta.author.is_empty() {
        out.push_str(&format!("作者：{}\n", novel.meta.author));
    }
    if !novel.meta.description.is_empty() {
        out.push_str(&format!("简介：{}\n", novel.meta.description));
    }
    out.push('\n');
    for vol in &novel.volumes {
        let vol_chapters: Vec<_> = chapters.iter().filter(|(id, _, _)| vol.chapters.iter().any(|c| &c.id == id)).collect();
        if vol_chapters.is_empty() {
            continue;
        }
        if vol.title != "正文" || vol.chapters.len() > 0 {
            out.push_str(&format!("\n{}\n{}\n\n", "=".repeat(20), vol.title));
        }
        for (_, title, text) in vol_chapters {
            out.push_str(&format!("\n{}\n{}\n\n", title, "-".repeat(12)));
            out.push_str(text);
            if !text.ends_with('\n') {
                out.push('\n');
            }
        }
    }
    std::fs::write(path, out).map_err(|e| format!("写入文件失败: {}", e))
}

/// 导出 Markdown
pub fn export_md(novel: &Novel, chapters: &[(String, String, String)], path: &Path) -> Result<(), String> {
    let mut out = String::new();
    out.push_str(&format!("# {}\n\n", novel.meta.title));
    if !novel.meta.author.is_empty() {
        out.push_str(&format!("> 作者：{}\n\n", novel.meta.author));
    }
    if !novel.meta.description.is_empty() {
        out.push_str(&format!("{}\n\n---\n", novel.meta.description));
    }
    for vol in &novel.volumes {
        let vol_chapters: Vec<_> = chapters.iter().filter(|(id, _, _)| vol.chapters.iter().any(|c| &c.id == id)).collect();
        if vol_chapters.is_empty() {
            continue;
        }
        if vol.title != "正文" {
            out.push_str(&format!("\n## {}\n\n", vol.title));
        }
        for (_, title, text) in vol_chapters {
            out.push_str(&format!("\n### {}\n\n", title));
            out.push_str(text);
            if !text.ends_with('\n') {
                out.push('\n');
            }
        }
    }
    std::fs::write(path, out).map_err(|e| format!("写入文件失败: {}", e))
}

/// 导出 .jsb 加密备份（密码派生密钥，可跨机器恢复）
pub fn export_jsb(
    password: &str,
    novel: &Novel,
    chapters: &[(String, String, String)],
    path: &Path,
) -> Result<(), String> {
    let data = serde_json::json!({
        "app": "jinshu-rust",
        "version": 1,
        "novel": novel,
        "chapters": chapters.iter().map(|(id, title, text)| serde_json::json!({
            "id": id, "title": title, "text": text
        })).collect::<Vec<_>>(),
    });
    let json = serde_json::to_vec(&data).map_err(|e| e.to_string())?;
    let blob = crypto::encrypt_jsb(password, &json)?;
    std::fs::write(path, blob).map_err(|e| format!("写入文件失败: {}", e))
}

/// 从 .jsb 导入（返回新小说与章节内容）
pub fn import_jsb(password: &str, data: &[u8]) -> Result<(Novel, Vec<(String, String, String)>), String> {
    let json = crypto::decrypt_jsb(password, data)?;
    let v: serde_json::Value = serde_json::from_slice(&json).map_err(|e| format!("备份解析失败: {}", e))?;
    let novel: Novel = serde_json::from_value(v["novel"].clone()).map_err(|e| format!("小说数据解析失败: {}", e))?;
    let mut chapters = Vec::new();
    if let Some(arr) = v["chapters"].as_array() {
        for item in arr {
            let id = item["id"].as_str().unwrap_or("").to_string();
            let title = item["title"].as_str().unwrap_or("").to_string();
            let text = item["text"].as_str().unwrap_or("").to_string();
            chapters.push((id, title, text));
        }
    }
    Ok((novel, chapters))
}

/// 导入落地：将备份写入存储
pub fn import_to_store(store: &Store, novel: &mut Novel, chapters: &[(String, String, String)]) -> Result<String, String> {
    // 生成新的 id，避免与现有小说冲突
    novel.meta.id = crate::util::new_id();
    novel.meta.created_at = crate::util::now_ts();
    store.save_novel(novel)?;
    for (cid, _, text) in chapters {
        store.save_chapter(&novel.meta.id, cid, text)?;
    }
    Ok(novel.meta.id.clone())
}

/// 默认导出目录（安装目录 data/exports）
pub fn default_export_dir(store: &Store) -> std::path::PathBuf {
    let dir = store.data_dir.join("exports");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Novel;

    #[test]
    fn export_formats() {
        let mut n = Novel::new("导出测试", "我", "简介", "仙侠");
        n.add_volume("卷一");
        let vid = n.volumes[0].id.clone();
        let cid = n.add_chapter(Some(&vid), "第一章 开始");
        let chapters = vec![(cid.clone(), "第一章 开始".to_string(), "正文内容第一行。".to_string())];
        let dir = std::env::temp_dir().join("jinshu_test");
        std::fs::create_dir_all(&dir).unwrap();
        let md = dir.join("t.md");
        export_md(&n, &chapters, &md).unwrap();
        let md_text = std::fs::read_to_string(&md).unwrap();
        assert!(md_text.contains("# 导出测试"));
        assert!(md_text.contains("### 第一章 开始"));
        assert!(md_text.contains("正文内容第一行。"));
        let txt = dir.join("t.txt");
        export_txt(&n, &chapters, &txt).unwrap();
        let txt_text = std::fs::read_to_string(&txt).unwrap();
        assert!(txt_text.contains("《导出测试》"));
        assert!(txt_text.contains("第一章 开始"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
