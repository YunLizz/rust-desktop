//! 应用外壳：eframe App 实现、布局、标题栏、状态栏、编辑器区、命令面板、快捷键

use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, Frame, Id, Key, KeyboardShortcut, Layout, Margin,
    Modifiers, Rect, RichText, Sense, Stroke, Vec2,
};

use crate::app::{Activity, AppState, DialogKind, FindState};
use crate::editor;
use crate::theme;
use crate::util;
use crate::widgets;
use crate::{ai_panel, dialogs, settings as settings_view, views};

pub struct App {
    pub state: AppState,
    pub custom_titlebar: bool,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // **预加载字体**：必须在 AppState::new() 之前对 egui_ctx 安装 CJK 字体，
        // 避免 egui issue #5840（Windows 下 app_creator 中设置的字体在第一帧被底层重置）。
        // 这里先按默认"非衬线"预加载；update() 中 need_font_reload=true 时会再按用户设置覆盖。
        theme::install_fonts(&cc.egui_ctx, false);

        let state = AppState::new(cc);
        let custom_titlebar = std::env::var("JINSHU_NATIVE_TITLEBAR").is_err() && cfg!(windows);
        Self { state, custom_titlebar }
    }

    // ---------- 命令 ----------
    fn execute_command(&mut self, cmd: Command) {
        let state = &mut self.state;
        match cmd {
            Command::NewNovel => state.dialog = Some(DialogKind::NewNovel),
            Command::OpenLibrary => {
                state.activity = Activity::Library;
                state.library = state.store().list_novels();
            }
            Command::Save => {
                state.save_all();
                state.show_toast("已保存（加密写入本地）", true);
            }
            Command::Export => {
                if state.novel.is_some() {
                    state.dialog = Some(DialogKind::Export);
                } else {
                    state.show_toast("请先打开一部小说", false);
                }
            }
            Command::Import => state.dialog = Some(DialogKind::Import),
            Command::ToggleSidebar => state.sidebar_open = !state.sidebar_open,
            Command::ToggleAI => {
                state.ai_panel_open = !state.ai_panel_open;
                state.activity = Activity::Chapters;
            }
            Command::ToggleTheme => {
                state.settings.theme = if state.settings.theme == "dark" { "light".to_string() } else { "dark".to_string() };
                state.refresh_palette();
                let _ = state.settings_save();
            }
            Command::FontUp => {
                state.settings.editor.font_size = (state.settings.editor.font_size + 1.0).min(32.0);
            }
            Command::FontDown => {
                state.settings.editor.font_size = (state.settings.editor.font_size - 1.0).max(10.0);
            }
            Command::Find => {
                if state.novel.is_some() && state.active_tab.is_some() {
                    state.find = Some(FindState::new());
                }
            }
            Command::Replace => {
                if state.novel.is_some() && state.active_tab.is_some() {
                    state.find = Some(FindState {
                        replace_mode: true,
                        ..FindState::new()
                    });
                }
            }
            Command::GlobalSearch => {
                state.activity = Activity::Search;
                state.sidebar_open = true;
            }
            Command::NewChapter => {
                if state.novel.is_some() {
                    state.dialog = Some(DialogKind::NewChapter);
                }
            }
            Command::NewVolume => {
                if state.novel.is_some() {
                    state.dialog = Some(DialogKind::NewVolume);
                }
            }
            Command::AiContinue | Command::AiOutline | Command::AiPolish | Command::AiExpand
            | Command::AiSummary | Command::AiChapterOutline | Command::AiPlotIdeas
            | Command::AiLogicCheck | Command::AiFeedback | Command::AiConsistency
            | Command::AiCharacterCard | Command::AiWorld | Command::AiNaming | Command::AiSynopsis => {
                if !state.ai_panel_open {
                    state.ai_panel_open = true;
                }
                state.activity = Activity::Chapters;
                let action = crate::ai_panel::action_label(&cmd);
                if let Some((msgs, is_action)) = crate::ai_panel::build_action_msgs(state, &cmd) {
                    if is_action {
                        state.start_ai(&action, msgs);
                    } else {
                        // 自由对话式：只聚焦输入框
                        state.ai_msgs.push(crate::app::AiMsg {
                            role: "user".into(),
                            content: msgs[0].1.clone(),
                            action,
                        });
                    }
                }
            }
            Command::CloseTab => {
                if let Some(cid) = state.active_tab.clone() {
                    state.close_tab(&cid);
                }
            }
            Command::CloseNovel => state.close_novel(),
            Command::About => state.dialog = Some(DialogKind::About),
            Command::Stats => state.activity = Activity::Stats,
            Command::Settings => state.activity = Activity::Settings,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Command {
    NewNovel, OpenLibrary, Save, Export, Import,
    ToggleSidebar, ToggleAI, ToggleTheme, FontUp, FontDown,
    Find, Replace, GlobalSearch, NewChapter, NewVolume, CloseTab, CloseNovel,
    AiContinue, AiOutline, AiPolish, AiExpand, AiSummary, AiChapterOutline,
    AiPlotIdeas, AiLogicCheck, AiFeedback, AiConsistency, AiCharacterCard,
    AiWorld, AiNaming, AiSynopsis, Stats, Settings, About,
}

impl Command {
    pub fn all() -> Vec<(Command, &'static str, &'static str)> {
        vec![
            (Command::NewNovel, "新建小说", "创建一部新的作品"),
            (Command::OpenLibrary, "打开书库", "浏览并打开已有作品"),
            (Command::Save, "保存全部", "加密保存所有未保存修改"),
            (Command::Export, "导出作品", "导出为 txt / md / jsb 加密备份"),
            (Command::Import, "导入 .jsb 备份", "从加密备份恢复作品"),
            (Command::NewChapter, "新建章节", "在当前卷下新建章节"),
            (Command::NewVolume, "新建分卷", "创建新的卷"),
            (Command::CloseTab, "关闭当前标签页", ""),
            (Command::CloseNovel, "关闭当前作品", "返回书库"),
            (Command::Find, "查找", "在章节内查找（Ctrl+F）"),
            (Command::Replace, "查找替换", "在章节内查找并替换（Ctrl+H）"),
            (Command::GlobalSearch, "全局搜索", "跨章节搜索（Ctrl+Shift+F）"),
            (Command::AiContinue, "AI 续写", "根据当前章节末尾续写正文"),
            (Command::AiOutline, "AI 生成大纲", "根据简介生成全本大纲"),
            (Command::AiChapterOutline, "AI 生成细纲", "为当前章节生成写作细纲"),
            (Command::AiPolish, "AI 润色选中文本", ""),
            (Command::AiExpand, "AI 扩写选中文本", ""),
            (Command::AiSummary, "AI 生成章节摘要", ""),
            (Command::AiPlotIdeas, "AI 剧情提示", "获取 5 个剧情发展方向"),
            (Command::AiLogicCheck, "AI 逻辑检查", "检查当前章节的逻辑漏洞"),
            (Command::AiConsistency, "AI 一致性检查", "全稿人物/时间线/设定一致性"),
            (Command::AiFeedback, "AI 整稿评审", "5 维度的编辑级反馈"),
            (Command::AiCharacterCard, "AI 设计人物卡", ""),
            (Command::AiWorld, "AI 设计世界观", ""),
            (Command::AiNaming, "AI 起名", "人物 / 地点 / 书名候选"),
            (Command::AiSynopsis, "AI 生成简介", ""),
            (Command::ToggleSidebar, "切换侧边栏", "Ctrl+B"),
            (Command::ToggleAI, "切换 AI 助手面板", "Ctrl+J"),
            (Command::ToggleTheme, "切换深色/浅色主题", ""),
            (Command::FontUp, "增大编辑区字号", "Ctrl+="),
            (Command::FontDown, "减小编辑区字号", "Ctrl+-"),
            (Command::Stats, "写作统计", "字数趋势与创作数据"),
            (Command::Settings, "打开设置", "主题 / 编辑 / AI 服务 / 存储"),
            (Command::About, "关于锦书", ""),
        ]
    }
}

// ---------- eframe App ----------
impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // AI 流式输出轮询
        self.state.poll_ai(ctx);
        // 自动保存
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);
        if now - self.state.last_autosave > self.state.settings.autosave_secs.max(1) as f64 {
            if !self.state.dirty.is_empty() || self.state.meta_dirty {
                self.state.save_all();
            }
            self.state.last_autosave = now;
        }
        // toast 过期
        if let Some((_, t, _)) = &self.state.toast {
            if now - t > 4.0 {
                self.state.toast = None;
            }
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // 主题与字体
        if self.state.need_font_reload {
            theme::install_fonts(&ctx, self.state.settings.editor.font == "serif");
            self.state.need_font_reload = false;
        }
        theme::apply_visuals(&ctx, &self.state.pal, self.state.settings.ui_scale, 14.0);

        self.handle_shortcuts(&ctx);

        if let Some(e) = &self.state.init_error {
            egui::CentralPanel::default().show(ui, |ui| {
                ui.centered_and_justified(|ui| {
                    ui.label(RichText::new("初始化失败").size(18.0).color(self.state.pal.danger));
                    ui.label(e.clone());
                });
            });
            return;
        }

        // 标题栏（root 级切分，避免嵌套面板导致的布局异常）
        self.titlebar(ui, &ctx);
        // 状态栏
        egui::Panel::bottom("statusbar")
            .exact_size(26.0)
            .resizable(false)
            .frame(Frame::new().fill(self.state.pal.bg_chrome))
            .show(ui, |ui| self.statusbar(ui, &ctx));
        // 活动栏
        if !self.state.focus_mode {
            egui::Panel::left("activity")
                .exact_size(48.0)
                .resizable(false)
                .show_separator_line(false)
                .frame(Frame::new().fill(self.state.pal.bg_chrome))
                .show(ui, |ui| self.activity_bar(ui));
            // 左侧面板（随活动切换）
            let sidebar_should_show = matches!(
                self.state.activity,
                Activity::Chapters | Activity::Outline | Activity::Characters
                    | Activity::World | Activity::Timeline | Activity::Tasks | Activity::Search
            ) && self.state.novel.is_some();
            let mut sidebar_open = self.state.sidebar_open && sidebar_should_show;
            egui::Panel::left("sidebar")
                .default_size(self.state.settings.sidebar_width)
                .min_size(220.0)
                .max_size(560.0)
                .resizable(true)
                .show_separator_line(false)
                .frame(Frame::new().fill(self.state.pal.bg_panel))
                .show_collapsible(ui, &mut sidebar_open, |ui| {
                    let show = matches!(
                        self.state.activity,
                        Activity::Chapters | Activity::Outline | Activity::Characters
                            | Activity::World | Activity::Timeline | Activity::Tasks | Activity::Search
                    );
                    if show && self.state.novel.is_some() {
                        self.sidebar_content(ui);
                    } else {
                        ui.add_space(8.0);
                    }
                });
            if sidebar_should_show {
                self.state.sidebar_open = sidebar_open;
            }
        }

        // AI 助手面板（右侧）
        if self.state.novel.is_some() && !self.state.focus_mode {
            let mut ai_open = self.state.ai_panel_open;
            egui::Panel::right("aipanel")
                .default_size(self.state.settings.ai_panel_width)
                .min_size(280.0)
                .max_size(620.0)
                .resizable(true)
                .show_separator_line(true)
                .frame(Frame::new().fill(self.state.pal.bg_panel))
                .show_collapsible(ui, &mut ai_open, |ui| {
                    ai_panel::show(ui, &mut self.state);
                });
            self.state.ai_panel_open = ai_open;
        }

        // 中央区域
        egui::CentralPanel::default()
            .frame(Frame::new().fill(self.state.pal.bg_editor))
            .show(ui, |ui| {
                self.central(ui, &ctx);
            });

        // 覆盖层：命令面板 / 对话框 / toast
        if self.state.palette_open {
            self.palette(&ctx);
        }
        dialogs::show_dialog(&mut self.state, &ctx);
        self.toast(&ctx);
    }

    fn on_exit(&mut self) {
        self.state.save_all();
        let s = self.state.settings.clone();
        if let Some(store) = &self.state.store {
            let _ = store.save_settings(&s);
        }
    }
}

impl App {
    // ---------- 快捷键 ----------
    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let mut cmds: Vec<Command> = Vec::new();
        ctx.input_mut(|i| {
            let ctrl = i.modifiers.command;
            let shift = i.modifiers.shift;
            if i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::N)) {
                cmds.push(Command::NewNovel);
            }
            if i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::O)) {
                cmds.push(Command::OpenLibrary);
            }
            if i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::S)) {
                cmds.push(Command::Save);
            }
            if i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::F)) {
                cmds.push(if shift { Command::GlobalSearch } else { Command::Find });
            }
            if i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::H)) {
                cmds.push(Command::Replace);
            }
            if i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::P)) {
                cmds.push(Command::GlobalSearch);
                self.state.palette_open = true;
                self.state.palette_query.clear();
            }
            if i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::B)) {
                cmds.push(Command::ToggleSidebar);
            }
            if i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::J)) {
                cmds.push(Command::ToggleAI);
            }
            if i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::Equals))
                || i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::Plus))
            {
                cmds.push(Command::FontUp);
            }
            if i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::Minus)) {
                cmds.push(Command::FontDown);
            }
            if i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::W)) {
                cmds.push(Command::CloseTab);
            }
            if ctrl && i.key_pressed(Key::Enter) && !self.state.ai_input.trim().is_empty() {
                // AI 输入框快捷键在 ai_panel 内处理
                let _ = i;
            }
        });
        if self.state.palette_open {
            ctx.input(|i| {
                if i.key_pressed(Key::Escape) {
                    self.state.palette_open = false;
                }
            });
            // 命令面板打开时不处理其余快捷键
            return;
        }
        for c in cmds {
            self.execute_command(c);
        }
    }

    // ---------- 标题栏 ----------
    fn titlebar(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let pal = &self.state.pal;
        let custom = self.custom_titlebar;
                let height = 40.0;
        egui::Panel::top("titlebar")
            .exact_size(height)
            .frame(Frame::new().fill(pal.bg_chrome))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let controls_w = if custom { 120.0 } else { 0.0 };
                    let (rect, resp) = ui.allocate_exact_size(
                        Vec2::new((ui.available_width() - controls_w).max(100.0), height),
                        Sense::drag(),
                    );
                    let _ = rect;
                    if custom {
                        if resp.dragged() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                        }
                        if resp.double_clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!ui.input(|i| i.viewport().maximized).unwrap_or(false)));
                        }
                    }
                    // 左区：应用名 + 作品名
                    ui.painter().text(
                        egui::pos2(rect.left() + 14.0, rect.center().y),
                        Align2::LEFT_CENTER,
                        "📖",
                        egui::FontId::proportional(15.0),
                        pal.accent,
                    );
                    ui.painter().text(
                        egui::pos2(rect.left() + 40.0, rect.center().y),
                        Align2::LEFT_CENTER,
                        "锦书",
                        egui::FontId::proportional(14.5),
                        pal.text,
                    );
                    if let Some(n) = &self.state.novel {
                        ui.painter().text(
                            egui::pos2(rect.left() + 92.0, rect.center().y),
                            Align2::LEFT_CENTER,
                            "·",
                            egui::FontId::proportional(13.0),
                            pal.text_disabled,
                        );
                        ui.painter().text(
                            egui::pos2(rect.left() + 108.0, rect.center().y),
                            Align2::LEFT_CENTER,
                            &n.meta.title,
                            egui::FontId::proportional(13.0),
                            pal.text_secondary,
                        );
                    }
                    // 未保存指示
                    if !self.state.dirty.is_empty() {
                        ui.painter().circle_filled(
                            egui::pos2(rect.left() + 30.0, rect.top() + 10.0),
                            4.0,
                            pal.warn,
                        );
                    }
                    // 窗口控制按钮（仅自定义标题栏时绘制，自绘图形避免字体缺字形）
                    if custom {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if win_btn(ui, WinBtn::Close, pal).clicked() {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                            ui.add_space(2.0);
                            let maximized = ui.input(|i| i.viewport().maximized).unwrap_or(false);
                            let kind = if maximized { WinBtn::Restore } else { WinBtn::Maximize };
                            if win_btn(ui, kind, pal).clicked() {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
                            }
                            ui.add_space(2.0);
                            if win_btn(ui, WinBtn::Minimize, pal).clicked() {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                            }
                        });
                    }
                });
            });
    }

    // ---------- 状态栏 ----------
    fn statusbar(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
                let pal = &self.state.pal;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 14.0;
            ui.label(RichText::new("🔒 加密存储").size(11.0).color(pal.ok));
            if let Some(n) = &self.state.novel {
                ui.label(RichText::new(format!("📖 {}", n.meta.title)).size(11.0).color(pal.text_secondary));
                let words: u64 = n.chapters_all().iter().map(|c| c.words).sum();
                ui.label(RichText::new(format!("字数 {}", words)).size(11.0).color(pal.text_secondary));
                let today = util::today();
                let today_words = n.stats.get(&today).copied().unwrap_or(0);
                ui.label(RichText::new(format!("今日 +{}", today_words)).size(11.0).color(pal.warn));
                // 光标位置
                if let Some(cid) = &self.state.active_tab {
                    if let Some(text) = self.state.chapters.get(cid) {
                        if let Some(cc) = self.state.last_cursor.get(cid) {
                            let (line, col) = line_col(text, *cc);
                            ui.label(RichText::new(format!("行 {}，列 {}", line, col)).size(11.0).color(pal.text_disabled));
                        }
                    }
                }
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(RichText::new("UTF-8 · 中文写作").size(11.0).color(pal.text_disabled));
                ui.label(RichText::new(if self.state.ai_streaming { "✨ AI 生成中…" } else { "✨ AI 就绪" }).size(11.0).color(if self.state.ai_streaming { pal.accent } else { pal.text_disabled }));
                let fp = &self.state.key_fp;
                let fp_short = if fp.len() > 8 { format!("{}…{}", &fp[..4], &fp[fp.len()-4..]) } else { fp.clone() };
                ui.label(
                    RichText::new(format!("🔑 密钥指纹 {}", fp_short))
                        .size(11.0)
                        .color(pal.text_disabled),
                );
            });
        });
    }

    // ---------- 活动栏 ----------
    fn activity_bar(&mut self, ui: &mut egui::Ui) {
        let pal = self.state.pal;
        ui.add_space(6.0);
        for act in Activity::all() {
            let selected = self.state.activity == act
                && (act != Activity::Search || true);
            if widgets::activity_btn(ui, act.icon(), act.label(), selected, &pal).clicked() {
                self.state.activity = act;
                if act == Activity::Search {
                    self.state.sidebar_open = true;
                }
                if act == Activity::Library || act == Activity::Stats || act == Activity::Settings {
                    self.state.library = self.state.store().list_novels();
                }
                if act == Activity::Settings {
                    self.state.sidebar_open = false;
                }
                if act == Activity::Library {
                    self.state.sidebar_open = false;
                }
            }
            ui.add_space(2.0);
        }
    }

    // ---------- 侧边栏 ----------
    fn sidebar_content(&mut self, ui: &mut egui::Ui) {
        let pal = self.state.pal;
        // 面板头
        let act = self.state.activity;
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.label(
                RichText::new(act.label())
                    .size(13.0)
                    .color(pal.text)
                    .strong(),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(8.0);
                if act == Activity::Chapters && widgets::icon_btn(ui, widgets::IC_PLUS, "新建章节", &pal).clicked() {
                    self.state.form_title.clear();
                    self.state.dialog = Some(DialogKind::NewChapter);
                }
                if act == Activity::Outline && widgets::icon_btn(ui, widgets::IC_PLUS, "添加卷", &pal).clicked() {
                    self.state.form_title.clear();
                    self.state.dialog = Some(DialogKind::NewVolume);
                }
            });
        });
        ui.add_space(2.0);
        widgets::h_sep(ui, &pal);
        ui.add_space(2.0);
        match act {
            Activity::Chapters => views::chapters_panel(ui, &mut self.state),
            Activity::Outline => views::outline_panel(ui, &mut self.state),
            Activity::Characters => views::characters_panel(ui, &mut self.state),
            Activity::World => views::world_panel(ui, &mut self.state),
            Activity::Timeline => views::timeline_panel(ui, &mut self.state),
            Activity::Tasks => views::tasks_panel(ui, &mut self.state),
            Activity::Search => views::search_panel(ui, &mut self.state),
            _ => {}
        }
    }

    // ---------- 中央区域 ----------
    fn central(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let pal = self.state.pal;
        match self.state.activity {
            Activity::Library => {
                if self.state.novel.is_none() && self.state.library.is_empty() {
                    views::welcome_view(ui, &mut self.state);
                } else {
                    views::library_view(ui, &mut self.state);
                }
            }
            Activity::Stats => {
                views::stats_view(ui, &mut self.state);
            }
            Activity::Settings => {
                settings_view::show(ui, &mut self.state);
            }
            Activity::Characters => {
                if self.state.novel.is_some() {
                    views::character_detail(ui, &mut self.state);
                } else {
                    views::welcome_view(ui, &mut self.state);
                }
            }
            Activity::World => {
                if self.state.novel.is_some() {
                    views::location_detail(ui, &mut self.state);
                } else {
                    views::welcome_view(ui, &mut self.state);
                }
            }
            Activity::Timeline => {
                if self.state.novel.is_some() {
                    views::timeline_detail(ui, &mut self.state);
                } else {
                    views::welcome_view(ui, &mut self.state);
                }
            }
            Activity::Tasks => {
                if self.state.novel.is_some() {
                    views::task_board(ui, &mut self.state);
                } else {
                    views::welcome_view(ui, &mut self.state);
                }
            }
            Activity::Outline => {
                if self.state.novel.is_some() {
                    views::outline_detail(ui, &mut self.state);
                } else {
                    views::welcome_view(ui, &mut self.state);
                }
            }
            _ => {
                if self.state.novel.is_none() {
                    views::welcome_view(ui, &mut self.state);
                } else if self.state.active_tab.is_none() {
                    widgets::empty_state(
                        ui,
                        "📑",
                        "还没有打开章节",
                        "在左侧「章节」面板选择或新建一个章节",
                        &pal,
                    );
                } else {
                    self.editor_view(ui, ctx);
                }
            }
        }
    }

    // ---------- 编辑器 ----------
    fn editor_view(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let pal = self.state.pal;
        let Some(novel) = self.state.novel.as_ref() else { return };
        let Some(cid) = self.state.active_tab.clone() else { return };
        let Some(cmeta) = novel.find_chapter(&cid).cloned() else {
            // 章节已被删除，关闭标签
            self.state.open_tabs.retain(|c| c != &cid);
            self.state.active_tab = self.state.open_tabs.last().cloned();
            return;
        };
        let focus_mode = self.state.focus_mode;

        // ---- 标签条 ----
        if !focus_mode {
            egui::Panel::top("tabs")
                .exact_size(36.0)
                .frame(Frame::new().fill(pal.bg_editor))
                .show(ui, |ui| self.tab_bar(ui));
        }

        // ---- 章节头 ----
        egui::Panel::top("chapter_header")
            .exact_size(44.0)
            .frame(Frame::new().fill(pal.bg_editor))
            .show(ui, |ui| {
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    let mut title = cmeta.title.clone();
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut title)
                            .font(egui::TextStyle::Heading)
                            .text_color(pal.text)
                            .desired_width(320.0)
                            .background_color(pal.bg_panel_alt)
                            .margin(Margin::symmetric(10, 5))
                            
                    );
                    if resp.changed() {
                        let t = title.trim().to_string();
                        if !t.is_empty() {
                            if let Some(c) = self.state.novel.as_mut().and_then(|n| n.find_chapter_mut(&cid)) {
                                c.title = t;
                            }
                            self.state.meta_dirty = true;
                        }
                    }
                    ui.label(
                        RichText::new(format!("{} 字", cmeta.words))
                            .size(12.0)
                            .color(pal.text_secondary),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        // AI 快捷操作
                        if widgets::secondary_btn(ui, "✨ 续写", &pal).clicked() {
                            let msgs = crate::ai_panel::build_continue_msgs(&mut self.state, "");
                            self.state.start_ai("续写", msgs);
                        }
                        ui.add_space(6.0);
                        if widgets::secondary_btn(ui, "💾 保存", &pal).clicked() {
                            self.state.save_all();
                            self.state.show_toast("已保存（加密写入本地）", true);
                        }
                    });
                });
            });

        // ---- 查找替换条 ----
        if let Some(find) = self.state.find.as_mut() {
            egui::Panel::top("findbar")
                .exact_size(if find.replace_mode { 74.0 } else { 40.0 })
                .frame(Frame::new().fill(pal.bg_panel))
                .show(ui, |ui| self.find_bar(ui, &cid));
        }

        // ---- 编辑区 ----
        let style = self.state.editor_style();
        let state_id_salt = format!("jinshu_editor_{}", cid);
        let Some(text) = self.state.chapters.get_mut(&cid) else { return };
        let mut out = editor::show_editor(ui, &state_id_salt, text, &style, &pal);

        // 光标与选区记录
        if let Some(cr) = out.cursor_range {
            let sorted = cr.as_sorted_char_range();
            let primary = cr.primary.index.0;
            self.state.last_cursor.insert(cid.clone(), primary);
            if sorted.end.0 - sorted.start.0 > 0 {
                let s: String = text.chars().skip(sorted.start.0).take(sorted.end.0 - sorted.start.0).collect();
                self.state.selected_text = s;
            } else {
                self.state.selected_text.clear();
            }
        }
        self.state.editor_focused = out.response.has_focus();
        // 编辑器内直接按 Ctrl+F / Ctrl+H / Esc
        ctx.input_mut(|i| {
            if out.response.has_focus() {
                if i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::F)) {
                    self.state.find = Some(FindState::new());
                }
                if i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::H)) {
                    self.state.find = Some(FindState {
                        replace_mode: true,
                        ..FindState::new()
                    });
                }
            }
        });
        if out.changed {
            self.state.mark_dirty(&cid);
            out.response.mark_changed();
        }
    }

    // ---------- 标签条 ----------
    fn tab_bar(&mut self, ui: &mut egui::Ui) {
        let pal = self.state.pal;
        let tabs = self.state.open_tabs.clone();
        let active = self.state.active_tab.clone();
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            ui.add_space(4.0);
            for cid in &tabs {
                let Some(novel) = self.state.novel.as_ref() else { continue };
                let Some(cmeta) = novel.find_chapter(cid) else { continue };
                let is_active = active.as_deref() == Some(cid.as_str());
                let dirty = self.state.dirty.contains(cid);
                let (rect, resp) = ui.allocate_exact_size(
                    Vec2::new(150.0, 26.0),
                    Sense::click(),
                );
                let bg = if is_active {
                    pal.bg_panel_alt
                } else if resp.hovered() {
                    pal.bg_hover
                } else {
                    Color32::TRANSPARENT
                };
                ui.painter().rect_filled(rect, CornerRadius::same(6), bg);
                if is_active {
                    ui.painter().rect_filled(
                        Rect::from_min_size(rect.min, Vec2::new(rect.width(), 2.0)),
                        CornerRadius::same(1),
                        pal.accent,
                    );
                }
                let mut title = cmeta.title.clone();
                if title.chars().count() > 8 {
                    title = title.chars().take(8).collect::<String>() + "…";
                }
                if dirty {
                    title.push(' ');
                }
                ui.painter().text(
                    egui::pos2(rect.left() + 10.0, rect.center().y),
                    Align2::LEFT_CENTER,
                    title,
                    egui::FontId::proportional(12.5),
                    if is_active { pal.text } else { pal.text_secondary },
                );
                if dirty {
                    ui.painter().circle_filled(
                        egui::pos2(rect.right() - 12.0, rect.center().y),
                        3.0,
                        pal.warn,
                    );
                }
                if resp.clicked() {
                    self.state.active_tab = Some(cid.clone());
                    // 从实体视图切回编辑区
                    self.state.activity = Activity::Chapters;
                }
                resp.context_menu(|ui| {
                    if ui.button("保存").clicked() {
                        self.state.save_chapter_now(cid);
                        ui.close();
                    }
                    if ui.button("关闭").clicked() {
                        self.state.close_tab(cid);
                        ui.close();
                    }
                    if ui.button("关闭其他").clicked() {
                        let others: Vec<String> = self.state.open_tabs.clone();
                        for o in others {
                            if &o != cid {
                                self.state.close_tab(&o);
                            }
                        }
                        self.state.active_tab = Some(cid.clone());
                        ui.close();
                    }
                });
            }
            // 右侧留白
            ui.add_space(ui.available_width() - 8.0);
        });
    }

    // ---------- 查找替换 ----------
    fn find_bar(&mut self, ui: &mut egui::Ui, cid: &str) {
        let pal = self.state.pal;
        let Some(mut fs) = self.state.find.take() else { return };
        let text = self.state.chapters.get(cid).cloned().unwrap_or_default();
        let state_id = ui.make_persistent_id(format!("jinshu_editor_{}", cid));
        let mut close = false;

        ui.horizontal(|ui| {
            ui.label(RichText::new("🔍").size(13.0).color(pal.text_secondary));
            let q = ui.add(
                egui::TextEdit::singleline(&mut fs.query)
                    .hint_text("查找…")
                    .text_color(pal.text)
                    .desired_width(240.0)
                    .margin(Margin::symmetric(8, 4))
                    .background_color(pal.bg_editor),
            );
            if q.changed() {
                update_matches(&mut fs, &text);
            }
            let total = fs.matches.len();
            let cur = if total == 0 { 0 } else { fs.current % total + 1 };
            ui.label(
                RichText::new(if total == 0 { "无结果".to_string() } else { format!("{}/{}", cur, total) })
                    .size(12.0)
                    .color(if total == 0 { pal.danger } else { pal.text_secondary }),
            );
            let go_prev = widgets::secondary_btn(ui, "上一个", &pal).clicked() && !fs.matches.is_empty();
            let mut go_next = widgets::secondary_btn(ui, "下一个", &pal).clicked() && !fs.matches.is_empty();
            if q.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) && !fs.query.is_empty() {
                go_next = true;
            }
            if go_prev {
                fs.current = if fs.current == 0 { fs.matches.len() - 1 } else { fs.current - 1 };
                jump_to(&self.state, ui.ctx(), state_id, &fs);
            }
            if go_next {
                fs.current = (fs.current + 1) % fs.matches.len();
                jump_to(&self.state, ui.ctx(), state_id, &fs);
            }
            if widgets::secondary_btn(ui, if fs.replace_mode { "替换模式" } else { "替换" }, &pal).clicked() {
                fs.replace_mode = true;
            }
            if widgets::icon_btn(ui, "✕", "关闭", &pal).clicked() {
                close = true;
            }
        });
        if fs.replace_mode {
            ui.horizontal(|ui| {
                ui.add_space(24.0);
                ui.add(
                    egui::TextEdit::singleline(&mut fs.replace)
                        .hint_text("替换为…")
                        .text_color(pal.text)
                        .desired_width(240.0)
                        .margin(Margin::symmetric(8, 4))
                        .background_color(pal.bg_editor),
                );
                if widgets::secondary_btn(ui, "替换当前", &pal).clicked() {
                    replace_current(&mut self.state, cid, &mut fs);
                    let text2 = self.state.chapters.get(cid).cloned().unwrap_or_default();
                    update_matches(&mut fs, &text2);
                }
                if widgets::secondary_btn(ui, "全部替换", &pal).clicked() {
                    replace_all(&mut self.state, cid, &mut fs);
                    let text2 = self.state.chapters.get(cid).cloned().unwrap_or_default();
                    update_matches(&mut fs, &text2);
                    self.state.show_toast("替换完成", true);
                }
                if widgets::secondary_btn(ui, "收起", &pal).clicked() {
                    fs.replace_mode = false;
                }
            });
        }
        if close {
            self.state.find = None;
        } else {
            self.state.find = Some(fs);
        }
    }

    // ---------- 命令面板 ----------
    fn palette(&mut self, ctx: &egui::Context) {
        let pal = self.state.pal;
        let mut close = false;
        let mut execute: Option<Command> = None;
        egui::Area::new(Id::new("palette"))
            .order(egui::Order::Foreground)
            .anchor(Align2::CENTER_TOP, Vec2::new(0.0, 90.0))
            .show(ctx, |ui| {
                widgets::popup_frame(&pal).show(ui, |ui| {
                    ui.set_width(560.0);
                    let query = &mut self.state.palette_query;
                    ui.add(
                        egui::TextEdit::singleline(query)
                            .hint_text("输入命令… (Esc 关闭)")
                            .text_color(pal.text)
                            .desired_width(f32::INFINITY)
                            .margin(Margin::symmetric(12, 8))

                            .background_color(pal.bg_panel),
                    );
                    ui.add_space(4.0);
                    let q = query.to_lowercase();
                    let mut items: Vec<(Command, &'static str)> = Command::all()
                        .into_iter()
                        .filter(|(_, label, _)| {
                            q.is_empty() || fuzzy_match(&q, &label.to_lowercase())
                        })
                        .take(14)
                        .map(|(c, l, _)| (c, l))
                        .collect();
                    if items.is_empty() {
                        items.push((Command::About, "没有匹配的命令"));
                    }
                    egui::ScrollArea::vertical()
                        .id_salt("palette_list")
                        .max_height(360.0)
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            for (cmd, label) in items {
                                let (rect, resp) = ui.allocate_exact_size(
                                    Vec2::new(ui.available_width(), 30.0),
                                    Sense::click(),
                                );
                                if resp.hovered() {
                                    ui.painter().rect_filled(rect, CornerRadius::same(5), pal.bg_hover);
                                }
                                ui.painter().text(
                                    egui::pos2(rect.left() + 10.0, rect.center().y),
                                    Align2::LEFT_CENTER,
                                    label,
                                    egui::FontId::proportional(13.0),
                                    pal.text,
                                );
                                if resp.clicked() {
                                    execute = Some(cmd);
                                    close = true;
                                }
                            }
                        });
                    if ui.input(|i| i.key_pressed(Key::Enter)) {
                        // 回车执行第一个
                        let first = Command::all()
                            .into_iter()
                            .find(|(_, l, _)| q.is_empty() || fuzzy_match(&q, &l.to_lowercase()));
                        if let Some((c, _, _)) = first {
                            execute = Some(c);
                            close = true;
                        }
                    }
                });
            });
        if close {
            self.state.palette_open = false;
        }
        if let Some(c) = execute {
            self.execute_command(c);
        }
    }

    // ---------- toast ----------
    fn toast(&mut self, ctx: &egui::Context) {
        let Some((msg, t, ok)) = self.state.toast.clone() else { return };
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);
        if now - t > 4.0 {
            self.state.toast = None;
            return;
        }
        let pal = self.state.pal;
        let alpha = (1.0 - ((now - t - 3.0).max(0.0))).clamp(0.0, 1.0);
        let bg = if ok { pal.ok } else { pal.danger };
        egui::Area::new(Id::new("toast"))
            .order(egui::Order::Foreground)
            .anchor(Align2::RIGHT_BOTTOM, Vec2::new(-24.0, -44.0))
            .show(ctx, |ui| {
                Frame::new()
                    .fill(bg.gamma_multiply(0.92 * alpha as f32))

                    .inner_margin(Margin::symmetric(14, 8))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(msg)
                                .size(12.5)
                                .color(Color32::from_rgb(250, 250, 252)),
                        );
                    });
            });
    }
}

// ---------- 窗口控制按钮（自绘） ----------
#[derive(Clone, Copy, Debug)]
enum WinBtn { Minimize, Maximize, Restore, Close }

fn win_btn(ui: &mut egui::Ui, kind: WinBtn, pal: &crate::widgets::Palette) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(28.0), Sense::click());
    let hover = resp.hovered();
    let bg = if hover {
        match kind {
            WinBtn::Close => pal.danger.gamma_multiply(0.30),
            _ => pal.bg_hover,
        }
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, CornerRadius::same(5), bg);
    let c = if hover { pal.text } else { pal.text_secondary };
    let center = rect.center();
    let s = 8.0;
    let stroke = Stroke::new(1.5, c);
    match kind {
        WinBtn::Minimize => {
            ui.painter().line_segment(
                [egui::pos2(center.x - s, center.y + 1.0), egui::pos2(center.x + s, center.y + 1.0)],
                stroke,
            );
        }
        WinBtn::Maximize => {
            let r = Rect::from_center_size(center, Vec2::new(s * 2.0 - 2.0, s * 2.0 - 4.0));
            ui.painter().rect_stroke(r, 1.0, stroke, egui::StrokeKind::Middle);
        }
        WinBtn::Restore => {
            let r1 = Rect::from_center_size(center + Vec2::new(3.0, -2.0), Vec2::new(s * 2.0 - 6.0, s * 2.0 - 8.0));
            ui.painter().rect_stroke(r1, 1.0, stroke, egui::StrokeKind::Middle);
            let r2 = Rect::from_center_size(center + Vec2::new(-3.0, 2.0), Vec2::new(s * 2.0 - 6.0, s * 2.0 - 8.0));
            ui.painter().rect_stroke(r2, 1.0, stroke, egui::StrokeKind::Middle);
        }
        WinBtn::Close => {
            ui.painter().line_segment(
                [center + Vec2::new(-s, -s + 2.0), center + Vec2::new(s, s - 2.0)],
                stroke,
            );
            ui.painter().line_segment(
                [center + Vec2::new(s, -s + 2.0), center + Vec2::new(-s, s - 2.0)],
                stroke,
            );
        }
    }
    let tip = match kind {
        WinBtn::Minimize => "最小化",
        WinBtn::Maximize => "最大化",
        WinBtn::Restore => "还原",
        WinBtn::Close => "关闭",
    };
    resp.on_hover_text(tip)
}

// ---------- 工具函数 ----------
fn ctx_of(ui: &egui::Ui) -> &egui::Context {
    ui.ctx()
}

fn line_col(text: &str, char_idx: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (i, c) in text.chars().enumerate() {
        if i >= char_idx {
            break;
        }
        if c == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

fn update_matches(fs: &mut FindState, text: &str) {
    fs.matches.clear();
    fs.current = 0;
    if fs.query.is_empty() {
        return;
    }
    let mut byte_pos = 0;
    for (byte_off, _) in text.match_indices(&fs.query) {
        let char_idx = text[..byte_off].chars().count();
        fs.matches.push(char_idx);
        byte_pos = byte_off;
        let _ = byte_pos;
    }
}

fn jump_to(_state: &AppState, ctx: &egui::Context, state_id: egui::Id, fs: &FindState) {
    if let Some(&idx) = fs.matches.get(fs.current % fs.matches.len().max(1)) {
        editor::set_cursor(ctx, state_id, idx);
    }
}

fn replace_current(state: &mut AppState, cid: &str, fs: &FindState) {
    let Some(text) = state.chapters.get_mut(cid) else { return };
    if fs.matches.is_empty() || fs.query.is_empty() {
        return;
    }
    let idx = fs.matches[fs.current % fs.matches.len()];
    if let Some(byte_off) = text.char_indices().nth(idx).map(|(b, _)| b) {
        let byte_end = byte_off + fs.query.len();
        if byte_end <= text.len() && &text[byte_off..byte_end] == fs.query {
            text.replace_range(byte_off..byte_end, &fs.replace);
            state.mark_dirty(cid);
        }
    }
}

fn replace_all(state: &mut AppState, cid: &str, fs: &mut FindState) {
    let Some(text) = state.chapters.get_mut(cid) else { return };
    if fs.query.is_empty() {
        return;
    }
    *text = text.replace(&fs.query, &fs.replace);
    state.mark_dirty(cid);
    fs.matches.clear();
    fs.current = 0;
}

fn fuzzy_match(query: &str, label: &str) -> bool {
    let mut qi = query.chars();
    let mut cur = qi.next();
    for c in label.chars() {
        if let Some(q) = cur {
            if q == c {
                cur = qi.next();
            }
        } else {
            break;
        }
    }
    cur.is_none()
}
