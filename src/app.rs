//! 应用核心：状态、布局、编辑器视图、命令面板、快捷键、自动保存

use std::collections::{HashMap, HashSet};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Receiver;
use std::sync::Arc;

use eframe::egui::{self, FontId};

use crate::ai::client::AiEvent;
use crate::editor::EditorStyle;
use crate::model::{Novel, NovelMeta};
use crate::store::Store;
use crate::theme::Palette;
use crate::util;
use crate::widgets;

// ---------- 导航 ----------
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Activity {
    Library,
    Chapters,
    Outline,
    Characters,
    World,
    Timeline,
    Tasks,
    Search,
    Stats,
    Settings,
}

impl Activity {
    pub fn icon(self) -> &'static str {
        match self {
            Activity::Library => widgets::IC_EXPLORER,
            Activity::Chapters => "📑",
            Activity::Outline => widgets::IC_OUTLINE,
            Activity::Characters => widgets::IC_PEOPLE,
            Activity::World => widgets::IC_MAP,
            Activity::Timeline => "⏱️",
            Activity::Tasks => widgets::IC_TASKS,
            Activity::Search => widgets::IC_SEARCH,
            Activity::Stats => widgets::IC_STATS,
            Activity::Settings => widgets::IC_SETTINGS,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Activity::Library => "书库",
            Activity::Chapters => "章节",
            Activity::Outline => "大纲",
            Activity::Characters => "人物",
            Activity::World => "世界观",
            Activity::Timeline => "时间线",
            Activity::Tasks => "任务",
            Activity::Search => "搜索",
            Activity::Stats => "统计",
            Activity::Settings => "设置",
        }
    }
    pub fn all() -> [Activity; 10] {
        [
            Activity::Library,
            Activity::Chapters,
            Activity::Outline,
            Activity::Characters,
            Activity::World,
            Activity::Timeline,
            Activity::Tasks,
            Activity::Search,
            Activity::Stats,
            Activity::Settings,
        ]
    }
}

// ---------- AI 消息 ----------
#[derive(Clone)]
pub struct AiMsg {
    pub role: String, // user | assistant | error
    pub content: String,
    pub action: String, // 用于按钮判断
}

// ---------- 查找 ----------
pub struct FindState {
    pub query: String,
    pub replace: String,
    pub matches: Vec<usize>, // char 索引
    pub current: usize,
    pub replace_mode: bool,
}

impl FindState {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            replace: String::new(),
            matches: Vec::new(),
            current: 0,
            replace_mode: false,
        }
    }
}

// ---------- 对话框 ----------
#[derive(PartialEq, Clone)]
pub enum DialogKind {
    NewNovel,
    NewChapter,
    NewVolume,
    RenameChapter(String),
    RenameVolume(String),
    DeleteChapter(String, String), // cid, title
    DeleteVolume(String, String),
    DeleteNovel(String, String),
    Export,
    Import,
    About,
}

// ---------- 应用状态 ----------
pub struct AppState {
    pub store: Option<Store>,
    pub settings: crate::store::AppSettings,
    pub pal: Palette,
    pub init_error: Option<String>,

    pub library: Vec<NovelMeta>,
    pub novel: Option<Novel>,
    pub chapters: HashMap<String, String>, // cid -> 正文（内存态）
    pub dirty: HashSet<String>,            // 未保存章节
    pub open_tabs: Vec<String>,
    pub active_tab: Option<String>,
    pub last_cursor: HashMap<String, usize>, // cid -> 光标字符位置
    pub selected_text: String,
    pub editor_focused: bool,

    pub activity: Activity,
    pub sidebar_open: bool,
    pub ai_panel_open: bool,
    pub focus_mode: bool,
    pub find: Option<FindState>,
    pub palette_open: bool,
    pub palette_query: String,

    // AI
    pub ai_msgs: Vec<AiMsg>,
    pub ai_input: String,
    pub ai_streaming: bool,
    pub ai_stream_text: String,
    pub ai_cancel: Arc<AtomicBool>,
    pub ai_rx: Option<Receiver<AiEvent>>,
    pub ai_action: String, // 当前动作名
    pub ai_error: Option<String>,
    pub ai_input_open: bool,
    pub ai_test_rx: Option<std::sync::mpsc::Receiver<Result<String, String>>>,
    pub summaries: HashMap<String, String>, // cid -> 摘要
    pub use_lore: bool,

    // 对话框状态
    pub dialog: Option<DialogKind>,
    pub form_title: String,
    pub form_author: String,
    pub form_genre: String,
    pub form_desc: String,
    pub export_fmt: String,
    pub jsb_pwd: String,
    pub jsb_pwd2: String,

    // 杂项
    pub toast: Option<(String, f64, bool)>,
    pub last_autosave: f64,
    pub meta_dirty: bool,
    pub need_font_reload: bool,
    pub data_dir: String,
    pub key_fp: String,
    // 人物关系画布
    pub canvas_pos: HashMap<String, egui::Pos2>,
    pub canvas_dragging: Option<(String, egui::Pos2, egui::Pos2)>,
    // 实体选择
    pub sel_char: Option<String>,
    pub sel_loc: Option<String>,
    pub sel_event: Option<String>,
    pub sel_outline: Option<String>,
    pub sel_chain: Option<String>,
    pub new_name: String,
    pub show_relation_canvas: bool,
}

impl AppState {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (store, init_error) = match Store::init() {
            Ok(s) => (Some(s), None),
            Err(e) => (None, Some(e)),
        };
        let settings = store
            .as_ref()
            .map(|s| s.load_settings())
            .unwrap_or_default();
        let (data_dir, key_fp) = store
            .as_ref()
            .map(|s| (s.data_dir.display().to_string(), s.key_fingerprint()))
            .unwrap_or_default();

        let mut st = Self {
            store,
            settings,
            pal: Palette::dark([0x6E, 0x8B, 0xFF]),
            init_error,
            library: Vec::new(),
            novel: None,
            chapters: HashMap::new(),
            dirty: HashSet::new(),
            open_tabs: Vec::new(),
            active_tab: None,
            last_cursor: HashMap::new(),
            selected_text: String::new(),
            editor_focused: false,
            activity: Activity::Library,
            sidebar_open: true,
            ai_panel_open: false,
            focus_mode: false,
            find: None,
            palette_open: false,
            palette_query: String::new(),
            ai_msgs: Vec::new(),
            ai_input: String::new(),
            ai_streaming: false,
            ai_stream_text: String::new(),
            ai_cancel: Arc::new(AtomicBool::new(false)),
            ai_rx: None,
            ai_action: String::new(),
            ai_error: None,
            ai_input_open: true,
            ai_test_rx: None,
            summaries: HashMap::new(),
            use_lore: true,
            dialog: None,
            form_title: String::new(),
            form_author: String::new(),
            form_genre: String::new(),
            form_desc: String::new(),
            export_fmt: "md".into(),
            jsb_pwd: String::new(),
            jsb_pwd2: String::new(),
            toast: None,
            last_autosave: 0.0,
            meta_dirty: false,
            need_font_reload: true,
            data_dir,
            key_fp,
            canvas_pos: HashMap::new(),
            canvas_dragging: None,
            sel_char: None,
            sel_loc: None,
            sel_event: None,
            sel_outline: None,
            sel_chain: None,
            new_name: String::new(),
            show_relation_canvas: false,
        };
        st.refresh_palette();
        if let Some(s) = &st.store {
            st.library = s.list_novels();
            // 恢复上次打开的小说
            if let Some(id) = st.settings.last_novel_id.clone() {
                st.open_novel(&id);
            }
        }
        st
    }

    pub fn store(&self) -> &Store {
        self.store.as_ref().expect("store initialized")
    }

    pub fn refresh_palette(&mut self) {
        self.pal = if self.settings.theme == "light" {
            Palette::light(self.settings.accent)
        } else {
            Palette::dark(self.settings.accent)
        };
    }

    pub fn show_toast(&mut self, msg: &str, ok: bool) {
        self.toast = Some((msg.to_string(), self.now(), ok));
    }
    fn now(&self) -> f64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0)
    }

    // ---------- 小说与章节 ----------
    pub fn open_novel(&mut self, id: &str) {
        let Some(store) = self.store.clone() else { return };
        match store.load_novel(id) {
            Ok(mut novel) => {
                novel.sync_from_chapters();
                self.save_novel_meta_silent(&mut novel); // 确保结构落盘
                // 加载全部章节
                let mut chapters = HashMap::new();
                for c in novel.chapters_all() {
                    if let Ok(t) = store.load_chapter(id, &c.id) {
                        chapters.insert(c.id.clone(), t);
                    }
                }
                self.novel = Some(novel);
                self.chapters = chapters;
                self.dirty.clear();
                self.open_tabs.clear();
                self.active_tab = None;
                self.find = None;
                self.ai_msgs.clear();
                self.summaries.clear();
                self.settings.last_novel_id = Some(id.to_string());
                let _ = self.settings_save();
                self.activity = Activity::Chapters;
                self.sidebar_open = true;
            }
            Err(e) => {
                self.show_toast(&format!("打开失败: {}", e), false);
            }
        }
    }

    pub fn close_novel(&mut self) {
        self.save_all();
        self.novel = None;
        self.chapters.clear();
        self.open_tabs.clear();
        self.active_tab = None;
        self.settings.last_novel_id = None;
        let _ = self.settings_save();
        self.library = self.store().list_novels();
        self.activity = Activity::Library;
    }

    pub fn active_text(&self) -> Option<&String> {
        let cid = self.active_tab.as_ref()?;
        self.chapters.get(cid)
    }

    pub fn active_text_mut(&mut self) -> Option<&mut String> {
        let cid = self.active_tab.clone()?;
        self.chapters.get_mut(&cid)
    }

    pub fn open_tab(&mut self, cid: &str) {
        if !self.open_tabs.iter().any(|c| c == cid) {
            self.open_tabs.push(cid.to_string());
        }
        self.active_tab = Some(cid.to_string());
    }

    /// 从书库导出前先打开作品（保留当前界面状态）
    pub fn open_novel_before_export(&mut self, id: &str) -> bool {
        self.open_novel(id);
        self.novel.is_some()
    }

    pub fn close_tab(&mut self, cid: &str) {
        self.save_chapter_now(cid);
        self.open_tabs.retain(|c| c != cid);
        if self.active_tab.as_deref() == Some(cid) {
            self.active_tab = self.open_tabs.last().cloned();
        }
        self.last_cursor.remove(cid);
    }

    pub fn mark_dirty(&mut self, cid: &str) {
        self.dirty.insert(cid.to_string());
        self.meta_dirty = true;
        if let Some(c) = self.novel.as_mut().and_then(|n| n.find_chapter_mut(cid)) {
            c.updated_at = util::now_ts();
        }
    }

    // ---------- 保存 ----------
    pub fn save_chapter_now(&mut self, cid: &str) {
        let Some(store) = self.store.clone() else { return };
        let Some(novel) = self.novel.as_mut() else { return };
        let Some(novel_id) = Some(novel.meta.id.clone()) else { return };
        let Some(text) = self.chapters.get(cid).cloned() else { return };
        let Some(cmeta) = novel.find_chapter_mut(cid) else { return };

        let old_words = cmeta.words;
        let new_words = util::count_words(&text);
        let delta = new_words.saturating_sub(old_words);
        cmeta.words = new_words;
        cmeta.updated_at = util::now_ts();
        let today = util::today();
        *novel.stats.entry(today).or_insert(0) += delta;
        novel.meta.total_words = novel.total_words();

        let ok = store.save_chapter(&novel_id, cid, &text).is_ok()
            && store.save_novel(novel).is_ok();
        if ok {
            self.dirty.remove(cid);
            self.meta_dirty = false;
        }
    }

    pub fn save_all(&mut self) {
        let cids: Vec<String> = self.chapters.keys().cloned().collect();
        for cid in cids {
            self.save_chapter_now(&cid);
        }
        if let Some(novel) = self.novel.as_mut() {
            novel.meta.updated_at = util::now_ts();
            novel.meta.total_words = novel.total_words();
            if let Some(store) = &self.store {
                let _ = store.save_novel(novel);
            }
            self.meta_dirty = false;
        }
    }

    pub fn save_novel_meta_silent(&self, novel: &Novel) {
        if let Some(store) = &self.store {
            let _ = store.save_novel(novel);
        }
    }

    pub fn settings_save(&mut self) -> Result<(), String> {
        let s = self.settings.clone();
        if let Some(store) = &self.store {
            store.save_settings(&s)
        } else {
            Ok(())
        }
    }

    // ---------- 编辑器样式 ----------
    pub fn editor_style(&self) -> EditorStyle {
        let ed = &self.settings.editor;
        let fam = if ed.font == "serif" {
            egui::FontFamily::Monospace
        } else {
            egui::FontFamily::Monospace
        };
        EditorStyle {
            font_id: FontId::new(ed.font_size, fam),
            line_spacing: ed.line_spacing,
            letter_spacing: if ed.font == "serif" { 0.6 } else { 0.0 },
            wrap: ed.wrap,
            md_highlight: ed.markdown_highlight,
            line_numbers: ed.show_line_numbers,
            gutter_bg: self.pal.bg_editor,
            text_color: self.pal.text,
            accent: self.pal.accent,
            hint_text: "开始创作吧……".into(),
        }
    }

    // ---------- AI ----------
    pub fn ai_busy(&self) -> bool {
        self.ai_streaming
    }

    pub fn start_ai(&mut self, action: &str, messages: Vec<(String, String)>) {
        if self.ai_streaming {
            self.show_toast("已有任务进行中，请先停止或等待", false);
            return;
        }
        let Some(store) = self.store.clone() else { return };
        let cfg = self.settings.ai.clone();
        if cfg.api_key.trim().is_empty() {
            self.show_toast("请先在「设置 → AI 服务」中填写 API Key", false);
            return;
        }
        let (tx, rx) = std::sync::mpsc::channel();
        self.ai_cancel = Arc::new(AtomicBool::new(false));
        let cancel = self.ai_cancel.clone();
        crate::ai::client::stream_chat(&cfg, &messages, tx, cancel);
        self.ai_rx = Some(rx);
        self.ai_streaming = true;
        self.ai_stream_text = String::new();
        self.ai_action = action.to_string();
        self.ai_error = None;
        // 用户消息入列
        let mut user_content = String::new();
        for (_, c) in &messages {
            user_content.push_str(c);
            user_content.push_str("\n\n");
        }
        self.ai_msgs.push(AiMsg {
            role: "user".into(),
            content: user_content.trim().to_string(),
            action: action.to_string(),
        });
        let _ = store;
    }

    pub fn poll_ai(&mut self, ctx: &egui::Context) {
        if !self.ai_streaming {
            return;
        }
        let Some(rx) = self.ai_rx.take() else { return };
        let mut done = false;
        let mut error: Option<String> = None;
        while let Ok(ev) = rx.try_recv() {
            match ev {
                AiEvent::Chunk(t) => self.ai_stream_text.push_str(&t),
                AiEvent::Done => done = true,
                AiEvent::Error(e) => error = Some(e),
            }
        }
        if done || error.is_some() {
            self.ai_streaming = false;
            let content = std::mem::take(&mut self.ai_stream_text);
            if let Some(e) = error {
                self.ai_msgs.push(AiMsg {
                    role: "error".into(),
                    content: e,
                    action: self.ai_action.clone(),
                });
            } else if !content.trim().is_empty() {
                self.ai_msgs.push(AiMsg {
                    role: "assistant".into(),
                    content,
                    action: self.ai_action.clone(),
                });
            }
            self.ai_action = String::new();
        } else if !self.ai_stream_text.is_empty() {
            // 还在流式输出，请求持续重绘
            self.ai_rx = Some(rx);
            ctx.request_repaint();
            return;
        } else {
            self.ai_rx = Some(rx);
            ctx.request_repaint();
            return;
        }
        self.ai_rx = None;
    }
}
