//! AI 助手面板：预设动作 + 流式对话 + 结果操作（插入正文 / 复制 / 应用大纲 / 设摘要）


use eframe::egui::{self, Align, CornerRadius, FontId, Layout, Margin, RichText, Sense, Vec2};

use crate::app::{AiMsg, AppState};
use crate::shell::Command;
use crate::util;
use crate::widgets::{self, Palette};

// ---------- 动作标签 ----------
pub fn action_label(cmd: &Command) -> String {
    match cmd {
        Command::AiContinue => "续写".into(),
        Command::AiOutline => "大纲生成".into(),
        Command::AiChapterOutline => "章节细纲".into(),
        Command::AiPolish => "润色".into(),
        Command::AiExpand => "扩写".into(),
        Command::AiSummary => "章节摘要".into(),
        Command::AiPlotIdeas => "剧情提示".into(),
        Command::AiLogicCheck => "逻辑检查".into(),
        Command::AiConsistency => "一致性检查".into(),
        Command::AiFeedback => "整稿评审".into(),
        Command::AiCharacterCard => "人物卡".into(),
        Command::AiWorld => "世界观".into(),
        Command::AiNaming => "起名".into(),
        Command::AiSynopsis => "简介".into(),
        _ => "对话".into(),
    }
}

// ---------- 提示词构建 ----------
pub fn build_continue_msgs(app: &mut AppState, instruction: &str) -> Vec<(String, String)> {
    let Some(novel) = app.novel.clone() else { return vec![] };
    let Some(cid) = app.active_tab.clone() else { return vec![] };
    let chapter_title = novel
        .find_chapter(&cid)
        .map(|c| c.title.clone())
        .unwrap_or_default();
    let text = app.chapters.get(&cid).cloned().unwrap_or_default();
    crate::ai::prompts::build_continue(&novel, &text, &chapter_title, instruction, app.use_lore)
}

pub fn build_outline_msgs(app: &mut AppState) -> Vec<(String, String)> {
    let Some(novel) = app.novel.clone() else { return vec![] };
    crate::ai::prompts::build_outline(
        &novel.meta.title,
        &novel.meta.description,
        &novel.meta.genre,
        "长篇（约 100 万字，100 卷规模）",
        "",
    )
}

pub fn build_character_msgs(app: &mut AppState, name: &str, role: &str) -> Vec<(String, String)> {
    let Some(novel) = app.novel.clone() else { return vec![] };
    crate::ai::prompts::build_character_card(&novel, name, role, "")
}

pub fn build_world_msgs(app: &mut AppState, name: &str, kind: &str) -> Vec<(String, String)> {
    let Some(novel) = app.novel.clone() else { return vec![] };
    let (name, kind) = if name.is_empty() {
        ("主世界", "世界观")
    } else {
        (name, kind)
    };
    crate::ai::prompts::build_world(&novel, name, kind, "")
}

/// 根据命令构造请求消息。返回 (消息, 是否立即发起 AI 调用)
pub fn build_action_msgs(app: &mut AppState, cmd: &Command) -> Option<(Vec<(String, String)>, bool)> {
    let novel = app.novel.clone()?;
    let cid = app.active_tab.clone();
    let chapter_text = cid
        .as_ref()
        .and_then(|c| app.chapters.get(c))
        .cloned()
        .unwrap_or_default();
    let chapter_title = cid
        .as_ref()
        .and_then(|c| novel.find_chapter(c))
        .map(|c| c.title.clone())
        .unwrap_or_default();

    match cmd {
        Command::AiContinue => Some((
            crate::ai::prompts::build_continue(&novel, &chapter_text, &chapter_title, "", app.use_lore),
            true,
        )),
        Command::AiOutline => Some((
            crate::ai::prompts::build_outline(
                &novel.meta.title,
                &novel.meta.description,
                &novel.meta.genre,
                "长篇",
                "",
            ),
            true,
        )),
        Command::AiChapterOutline => Some((
            crate::ai::prompts::build_chapter_outline(&novel, &chapter_text, &chapter_title),
            true,
        )),
        Command::AiPolish => {
            let sel = app.selected_text.clone();
            if sel.trim().is_empty() {
                app.show_toast("请先在正文中选中要润色的文本", false);
                return None;
            }
            Some((crate::ai::prompts::build_polish(&sel, ""), true))
        }
        Command::AiExpand => {
            let sel = app.selected_text.clone();
            if sel.trim().is_empty() {
                app.show_toast("请先在正文中选中要扩写的文本", false);
                return None;
            }
            Some((crate::ai::prompts::build_expand(&sel, ""), true))
        }
        Command::AiSummary => Some((
            crate::ai::prompts::build_summary(&chapter_title, &chapter_text),
            true,
        )),
        Command::AiPlotIdeas => Some((
            crate::ai::prompts::build_plot_ideas(&novel, &chapter_text, &chapter_title),
            true,
        )),
        Command::AiLogicCheck => Some((
            crate::ai::prompts::build_logic_check(&novel, &chapter_text, &chapter_title),
            true,
        )),
        Command::AiConsistency => {
            let summaries = chapter_summaries(app, &novel);
            Some((crate::ai::prompts::build_consistency(&novel, &summaries), true))
        }
        Command::AiFeedback => {
            let summaries = chapter_summaries(app, &novel);
            Some((crate::ai::prompts::build_feedback(&novel, &summaries), true))
        }
        Command::AiCharacterCard => {
            let name = if app.new_name.trim().is_empty() { "主角" } else { app.new_name.trim() };
            Some((crate::ai::prompts::build_character_card(&novel, name, "主角", ""), true))
        }
        Command::AiWorld => Some((crate::ai::prompts::build_world(&novel, "主世界", "世界观", ""), true)),
        Command::AiNaming => Some((crate::ai::prompts::build_naming(&novel, "古典、有仙气", "人物", 8), true)),
        Command::AiSynopsis => Some((crate::ai::prompts::build_synopsis(&novel), true)),
        _ => None,
    }
}

fn chapter_summaries(app: &AppState, novel: &crate::model::Novel) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for c in novel.chapters_all() {
        if let Some(s) = app.summaries.get(&c.id) {
            out.push((c.title.clone(), s.clone()));
        } else if let Some(t) = app.chapters.get(&c.id) {
            let head: String = t.chars().take(120).collect();
            out.push((c.title.clone(), format!("（无摘要，开头：{}…）", head)));
        }
    }
    out
}

// ---------- 面板 ----------
pub fn show(ui: &mut egui::Ui, app: &mut AppState) {
    let pal = app.pal;
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.add_space(6.0);
        ui.label(RichText::new("✨ AI 创作助手").size(13.5).strong().color(pal.text));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if widgets::icon_btn(ui, "✕", "关闭面板", &pal).clicked() {
                app.ai_panel_open = false;
            }
            let _ = widgets::toggle(ui, &mut app.use_lore, &pal);
            ui.label(RichText::new("注入设定").size(10.5).color(pal.text_disabled));
        });
    });
    ui.add_space(2.0);
    widgets::h_sep(ui, &pal);
    ui.add_space(4.0);

    // 预设动作
    egui::ScrollArea::horizontal().id_salt("ai_chips").show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 5.0;
            for (label, cmd) in [
                ("✍️ 续写", Command::AiContinue),
                ("📝 细纲", Command::AiChapterOutline),
                ("🧹 润色", Command::AiPolish),
                ("📐 扩写", Command::AiExpand),
                ("📄 摘要", Command::AiSummary),
                ("💡 剧情", Command::AiPlotIdeas),
                ("🔍 逻辑", Command::AiLogicCheck),
                ("🧬 一致", Command::AiConsistency),
                ("📋 评审", Command::AiFeedback),
                ("👤 人物卡", Command::AiCharacterCard),
                ("🌍 世界观", Command::AiWorld),
                ("📛 起名", Command::AiNaming),
                ("📢 简介", Command::AiSynopsis),
            ] {
                if widgets::pill(ui, label, false, &pal).clicked() {
                    if let Some((msgs, is_action)) = build_action_msgs(app, &cmd) {
                        if is_action {
                            app.start_ai(&action_label(&cmd), msgs);
                        }
                    }
                }
            }
        });
    });
    ui.add_space(4.0);
    widgets::h_sep(ui, &pal);
    ui.add_space(4.0);

    // 对话区
    let msgs = app.ai_msgs.clone();
    let streaming = app.ai_streaming;
    let stream_text = app.ai_stream_text.clone();
    let ai_action = app.ai_action.clone();

    egui::ScrollArea::vertical()
        .id_salt("ai_chat")
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            for (i, m) in msgs.iter().enumerate() {
                msg_bubble(ui, app, i, m);
            }
            if streaming {
                // 流式气泡
                let (rect, _) = ui.allocate_exact_size(
                    Vec2::new(ui.available_width(), 0.0),
                    Sense::hover(),
                );
                let _ = rect;
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(
                        RichText::new(format!("{} 生成中…", ai_action))
                            .size(11.5)
                            .color(pal.accent),
                    );
                });
                if !stream_text.is_empty() {
                    bubble(ui, &pal, &stream_text, false);
                }
            }
            ui.add_space(6.0);
        });

    // 输入区
    ui.add_space(2.0);
    let sendable = !app.ai_input.trim().is_empty();
    ui.horizontal(|ui| {
        let resp = ui.add(
            egui::TextEdit::multiline(&mut app.ai_input)
                .hint_text("自由提问，或对 AI 结果说「再短一点」…")
                .text_color(pal.text)
                .desired_width(ui.available_width() - 60.0)
                .desired_rows(2)
                .margin(Margin::symmetric(8, 6))
                
                .background_color(pal.bg_editor),
        );
        let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && !ui.input(|i| i.modifiers.shift);
        if enter && sendable {
            let input = std::mem::take(&mut app.ai_input);
            let (outline, chars, world) = {
                let novel = app.novel.clone().unwrap_or_default();
                let text = app.active_text().cloned().unwrap_or_default();
                crate::ai::prompts::build_context(&novel, &text, app.use_lore)
            };
            let p = format!(
                "自由提问（当前作品《{}》，已有大纲：{}；人物：{}；世界观：{}）：\n{}",
                app.novel.as_ref().map(|n| n.meta.title.clone()).unwrap_or_default(),
                util::truncate_chars(&outline, 800),
                util::truncate_chars(&chars, 800),
                util::truncate_chars(&world, 600),
                input
            );
            app.start_ai("对话", vec![("user".into(), p)]);
        }
        let _ = enter;
        if streaming {
            if widgets::icon_btn(ui, "⏹", "停止生成", &pal).clicked() {
                app.ai_cancel.store(true, std::sync::atomic::Ordering::Relaxed);
            }
        } else if widgets::icon_btn(ui, "➤", "发送", &pal).clicked() && sendable {
            let input = std::mem::take(&mut app.ai_input);
            let (outline, chars, world) = {
                let novel = app.novel.clone().unwrap_or_default();
                let text = app.active_text().cloned().unwrap_or_default();
                crate::ai::prompts::build_context(&novel, &text, app.use_lore)
            };
            let p = format!(
                "自由提问（当前作品《{}》，已有大纲：{}；人物：{}；世界观：{}）：\n{}",
                app.novel.as_ref().map(|n| n.meta.title.clone()).unwrap_or_default(),
                util::truncate_chars(&outline, 800),
                util::truncate_chars(&chars, 800),
                util::truncate_chars(&world, 600),
                input
            );
            app.start_ai("对话", vec![("user".into(), p)]);
        }
    });
}

fn bubble(ui: &mut egui::Ui, pal: &Palette, text: &str, user: bool) {
    let width = ui.available_width().max(40.0);
    let wrap = ui.painter().layout(
        text.to_string(),
        FontId::proportional(12.5),
        pal.text,
        width - 20.0,
    );
    let h = wrap.size().y + 16.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, h), Sense::hover());
    let bg = if user { pal.accent.gamma_multiply(0.22) } else { pal.bg_panel_alt };
    ui.painter().rect_filled(rect, CornerRadius::same(8), bg);
    ui.painter().galley(rect.min + Vec2::new(10.0, 8.0), wrap, pal.text);
}

fn msg_bubble(ui: &mut egui::Ui, app: &mut AppState, idx: usize, m: &AiMsg) {
    let pal = app.pal;
    if m.role == "user" {
        bubble(ui, &pal, &m.content, true);
        ui.add_space(2.0);
    } else if m.role == "error" {
        ui.horizontal(|ui| {
            ui.label(RichText::new("⚠️").color(pal.danger));
            ui.label(RichText::new(&m.content).size(12.0).color(pal.danger));
        });
        ui.add_space(4.0);
    } else {
        bubble(ui, &pal, &m.content, false);
        // 操作按钮
        ui.horizontal(|ui| {
            ui.add_space(6.0);
            let mut insert = false;
            let mut copy = false;
            let mut regen = false;
            let mut apply_outline = false;
            let mut set_summary = false;
            if widgets::secondary_btn(ui, "📥 插入到正文", &pal).clicked() {
                insert = true;
            }
            if widgets::secondary_btn(ui, "📋 复制", &pal).clicked() {
                copy = true;
            }
            if widgets::secondary_btn(ui, "🔄 重新生成", &pal).clicked() {
                regen = true;
            }
            if m.action.contains("大纲") && !m.content.trim().is_empty() {
                if widgets::secondary_btn(ui, "🗂 应用为大纲", &pal).clicked() {
                    apply_outline = true;
                }
            }
            if (m.action == "章节摘要" || m.action == "摘要") && !m.content.trim().is_empty() {
                if widgets::secondary_btn(ui, "📌 设为章节摘要", &pal).clicked() {
                    set_summary = true;
                }
            }
            if insert {
                insert_to_editor(app, &m.content);
            }
            if copy {
                ui.ctx().copy_text(m.content.clone());
                app.show_toast("已复制", true);
            }
            if regen {
                regenerate(app, idx);
            }
            if apply_outline {
                let nodes = crate::ai::prompts::parse_outline(&m.content);
                if !nodes.is_empty() {
                    if let Some(n) = app.novel.as_mut() {
                        n.outline = nodes;
                        app.meta_dirty = true;
                    }
                    app.show_toast("大纲已应用（可在「大纲」面板查看）", true);
                } else {
                    app.show_toast("未能从回复中解析出大纲结构", false);
                }
            }
            if set_summary {
                if let Some(cid) = app.active_tab.clone() {
                    let s: String = m.content.chars().take(400).collect();
                    app.summaries.insert(cid, s);
                    app.show_toast("已设为章节摘要（用于全稿检查）", true);
                }
            }
        });
        ui.add_space(6.0);
    }
}

fn insert_to_editor(app: &mut AppState, content: &str) {
    let Some(cid) = app.active_tab.clone() else {
        app.show_toast("请先打开一个章节", false);
        return;
    };
    let Some(text) = app.chapters.get_mut(&cid) else { return };
    let cursor = app.last_cursor.get(&cid).copied().unwrap_or_else(|| text.chars().count());
    let mut out: String = text.chars().take(cursor).collect();
    out.push_str("\n\n");
    out.push_str(content.trim());
    out.push('\n');
    out.push_str(&text.chars().skip(cursor).collect::<String>());
    *text = out;
    app.mark_dirty(&cid);
    app.show_toast("已插入到当前光标位置", true);
}

fn regenerate(app: &mut AppState, idx: usize) {
    if app.ai_streaming {
        return;
    }
    // 找到该消息之前的最近一条 user 消息，重新发起
    let msgs = app.ai_msgs.clone();
    let mut user_content = String::new();
    for m in msgs.iter().take(idx).rev() {
        if m.role == "user" {
            user_content = m.content.clone();
            break;
        }
    }
    if user_content.is_empty() {
        app.show_toast("没有可重新生成的请求", false);
        return;
    }
    // 截断此前的助手消息，保留到该用户消息
    app.ai_msgs.truncate(idx);
    app.start_ai("对话", vec![("user".into(), user_content)]);
}
