#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! 锦书 · Tauri 后端：存储 / 章节 / 设置 / AI 流式 / 导出导入

mod ai_client;
mod ai_prompts;
mod crypto;
mod export;
mod model;
mod store;
mod util;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

pub struct AppData {
    store: Mutex<store::Store>,
    cancel: Mutex<Option<Arc<AtomicBool>>>,
}

// ---------- 基础 ----------
#[derive(Serialize)]
struct InitInfo {
    data_dir: String,
    key_fp: String,
    settings: store::AppSettings,
}

#[tauri::command]
fn init_info(state: State<AppData>) -> Result<InitInfo, String> {
    let s = state.store.lock().map_err(|e| e.to_string())?;
    Ok(InitInfo {
        data_dir: s.data_dir.display().to_string(),
        key_fp: s.key_fingerprint(),
        settings: s.load_settings(),
    })
}

#[tauri::command]
fn list_novels(state: State<AppData>) -> Result<Vec<model::NovelMeta>, String> {
    let s = state.store.lock().map_err(|e| e.to_string())?;
    Ok(s.list_novels())
}

// ---------- 小说与章节 ----------
#[derive(Serialize)]
struct ChapterData {
    id: String,
    title: String,
    text: String,
}

#[derive(Serialize)]
struct NovelBundle {
    novel: model::Novel,
    chapters: Vec<ChapterData>,
}

#[tauri::command]
fn load_novel(state: State<AppData>, id: String) -> Result<NovelBundle, String> {
    let s = state.store.lock().map_err(|e| e.to_string())?;
    let mut novel = s.load_novel(&id)?;
    novel.sync_from_chapters();
    let _ = s.save_novel(&novel); // 确保结构落盘
    let mut chapters = Vec::new();
    for c in novel.chapters_all() {
        let text = s.load_chapter(&id, &c.id).unwrap_or_default();
        chapters.push(ChapterData {
            id: c.id.clone(),
            title: c.title.clone(),
            text,
        });
    }
    Ok(NovelBundle { novel, chapters })
}

#[tauri::command]
fn create_novel(
    state: State<AppData>,
    title: String,
    author: String,
    genre: String,
    description: String,
) -> Result<String, String> {
    let s = state.store.lock().map_err(|e| e.to_string())?;
    let mut novel = model::Novel::new(&title, &author, &description, &genre);
    novel.add_volume("正文");
    let vid = novel.volumes[0].id.clone();
    novel.add_chapter(Some(&vid), "第一章 开端");
    let id = novel.meta.id.clone();
    s.save_novel(&novel)?;
    Ok(id)
}

#[tauri::command]
fn save_novel(state: State<AppData>, novel: model::Novel) -> Result<(), String> {
    let s = state.store.lock().map_err(|e| e.to_string())?;
    s.save_novel(&novel)
}

/// 保存章节正文并同步统计；返回更新后的（章节字数、总字数、今日统计）
#[derive(Serialize)]
struct ChapterSaved {
    words: u64,
    total_words: u64,
    stats: std::collections::BTreeMap<String, u64>,
}

#[tauri::command]
fn save_chapter(
    state: State<AppData>,
    novel_id: String,
    cid: String,
    text: String,
) -> Result<ChapterSaved, String> {
    let s = state.store.lock().map_err(|e| e.to_string())?;
    let mut novel = s.load_novel(&novel_id)?;
    let old_words = novel
        .find_chapter(&cid)
        .map(|c| c.words)
        .unwrap_or(0);
    let new_words = util::count_words(&text);
    let delta = new_words.saturating_sub(old_words);
    if let Some(c) = novel.find_chapter_mut(&cid) {
        c.words = new_words;
        c.updated_at = util::now_ts();
    }
    let today = util::today();
    *novel.stats.entry(today).or_insert(0) += delta;
    novel.meta.total_words = novel.total_words();
    novel.meta.updated_at = util::now_ts();
    s.save_chapter(&novel_id, &cid, &text)?;
    s.save_novel(&novel)?;
    Ok(ChapterSaved {
        words: new_words,
        total_words: novel.meta.total_words,
        stats: novel.stats.clone(),
    })
}

#[tauri::command]
fn delete_chapter(state: State<AppData>, novel_id: String, cid: String) -> Result<(), String> {
    let s = state.store.lock().map_err(|e| e.to_string())?;
    let mut novel = s.load_novel(&novel_id)?;
    novel.delete_chapter(&cid);
    s.delete_chapter_file(&novel_id, &cid);
    s.save_novel(&novel)
}

#[tauri::command]
fn delete_novel(state: State<AppData>, id: String) -> Result<(), String> {
    let s = state.store.lock().map_err(|e| e.to_string())?;
    s.delete_novel(&id)
}

// ---------- 设置 ----------
#[tauri::command]
fn load_settings(state: State<AppData>) -> Result<store::AppSettings, String> {
    let s = state.store.lock().map_err(|e| e.to_string())?;
    Ok(s.load_settings())
}

#[tauri::command]
fn save_settings(state: State<AppData>, settings: store::AppSettings) -> Result<(), String> {
    let s = state.store.lock().map_err(|e| e.to_string())?;
    s.save_settings(&settings)
}

// ---------- 导出 / 导入 ----------
#[tauri::command]
fn export_work(
    state: State<AppData>,
    fmt: String,
    path: String,
    password: Option<String>,
    novel: model::Novel,
    chapters: Vec<(String, String, String)>,
) -> Result<(), String> {
    let _ = state;
    let p = std::path::PathBuf::from(path);
    match fmt.as_str() {
        "txt" => export::export_txt(&novel, &chapters, &p),
        "md" => export::export_md(&novel, &chapters, &p),
        "jsb" => {
            let pwd = password.unwrap_or_default();
            export::export_jsb(&pwd, &novel, &chapters, &p)
        }
        _ => Err("未知导出格式".into()),
    }
}

/// 导入 .jsb 备份，返回新小说 id
#[tauri::command]
fn import_jsb(
    state: State<AppData>,
    path: String,
    password: String,
) -> Result<String, String> {
    let s = state.store.lock().map_err(|e| e.to_string())?;
    let bytes = std::fs::read(&path).map_err(|e| format!("读取文件失败: {}", e))?;
    let (mut novel, chapters) = export::import_jsb(&password, &bytes)?;
    export::import_to_store(&s, &mut novel, &chapters)
}

#[tauri::command]
fn open_dir(path: String) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer").arg(&path).spawn();
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = std::process::Command::new("xdg-open").arg(&path).spawn();
    }
}

// ---------- AI ----------
/// 启动流式 AI 请求；结果通过事件推送: ai-chunk / ai-done / ai-error
#[tauri::command]
fn ai_start(
    app: AppHandle,
    state: State<AppData>,
    cfg: store::AiSettings,
    messages: Vec<(String, String)>,
) -> Result<(), String> {
    let cancel = Arc::new(AtomicBool::new(false));
    *state.cancel.lock().map_err(|e| e.to_string())? = Some(cancel.clone());
    let app2 = app.clone();
    std::thread::spawn(move || {
        let (tx, rx) = std::sync::mpsc::channel();
        ai_client::stream_chat(&cfg, &messages, tx, cancel);
        let mut done = false;
        while let Ok(ev) = rx.recv() {
            match ev {
                ai_client::AiEvent::Chunk(t) => {
                    let _ = app2.emit("ai-chunk", t);
                }
                ai_client::AiEvent::Done => {
                    done = true;
                    break;
                }
                ai_client::AiEvent::Error(e) => {
                    let _ = app2.emit("ai-error", e);
                    return;
                }
            }
        }
        if done {
            let _ = app2.emit("ai-done", ());
        }
    });
    Ok(())
}

#[tauri::command]
fn ai_cancel(state: State<AppData>) {
    if let Ok(mut c) = state.cancel.lock() {
        if let Some(f) = c.take() {
            f.store(true, Ordering::Relaxed);
        }
    }
}

/// 测试连接；结果通过事件 ai-test-result 推送
#[tauri::command]
fn ai_test(app: AppHandle, cfg: store::AiSettings) {
    let app2 = app.clone();
    std::thread::spawn(move || {
        let r = ai_client::test_connection(&cfg);
        let _ = app2.emit("ai-test-result", r);
    });
}

fn main() {
    let store = match store::Store::init() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("存储初始化失败: {}", e);
            std::process::exit(1);
        }
    };
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AppData {
            store: Mutex::new(store),
            cancel: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            init_info,
            list_novels,
            load_novel,
            create_novel,
            save_novel,
            save_chapter,
            delete_chapter,
            delete_novel,
            load_settings,
            save_settings,
            export_work,
            import_jsb,
            open_dir,
            ai_start,
            ai_cancel,
            ai_test,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
