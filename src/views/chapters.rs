//! 章节树 / 大纲面板

use eframe::egui::{self, Align2, CornerRadius, FontId, RichText, Sense, Vec2};

use crate::app::{AppState, DialogKind};
use crate::model::OutlineNode;
use crate::widgets;

// ---------- 章节树 ----------
pub fn chapters_panel(ui: &mut egui::Ui, app: &mut AppState) {
    let pal = app.pal;
    let Some(novel) = app.novel.as_ref() else { return };
    let volumes = novel.volumes.clone();

    let mut rename_vol: Option<String> = None;
    let mut delete_vol: Option<(String, String)> = None;
    let mut new_chapter_in: Option<String> = None;

    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        for vol in &volumes {
            let vol_words: u64 = vol.chapters.iter().map(|c| c.words).sum();
            let header = format!("📁 {}（{}章 · {}字）", vol.title, vol.chapters.len(), vol_words);
            egui::CollapsingHeader::new(RichText::new(header).size(12.5).color(pal.text))
                .id_salt(("vol", &vol.id))
                .default_open(true)
                .show(ui, |ui| {
                    for c in &vol.chapters {
                        let is_active = app.active_tab.as_deref() == Some(c.id.as_str());
                        let dirty = app.dirty.contains(&c.id);
                        let (rect, resp) = ui.allocate_exact_size(
                            Vec2::new(ui.available_width(), 26.0),
                            Sense::click(),
                        );
                        if is_active {
                            ui.painter().rect_filled(
                                rect,
                                CornerRadius::same(5),
                                pal.accent.gamma_multiply(0.14),
                            );
                        } else if resp.hovered() {
                            ui.painter().rect_filled(rect, CornerRadius::same(5), pal.bg_hover);
                        }
                        let mut title = c.title.clone();
                        if title.chars().count() > 12 {
                            title = title.chars().take(12).collect::<String>() + "…";
                        }
                        let dot = if dirty { " ●" } else { "" };
                        ui.painter().text(
                            egui::pos2(rect.left() + 16.0, rect.center().y),
                            Align2::LEFT_CENTER,
                            format!("📄 {}{}", title, dot),
                            FontId::proportional(12.5),
                            if is_active { pal.text } else { pal.text_secondary },
                        );
                        ui.painter().text(
                            egui::pos2(rect.right() - 8.0, rect.center().y),
                            Align2::RIGHT_CENTER,
                            format!("{}字", c.words),
                            FontId::proportional(10.0),
                            pal.text_disabled,
                        );
                        if resp.clicked() {
                            app.open_tab(&c.id);
                            app.activity = crate::app::Activity::Chapters;
                        }
                        resp.context_menu(|ui| {
                            if ui.button("打开").clicked() {
                                app.open_tab(&c.id);
                                ui.close();
                            }
                            if ui.button("重命名").clicked() {
                                app.dialog = Some(DialogKind::RenameChapter(c.id.clone()));
                                ui.close();
                            }
                            if ui.button("上移").clicked() {
                                move_chapter(app, &c.id, -1);
                                ui.close();
                            }
                            if ui.button("下移").clicked() {
                                move_chapter(app, &c.id, 1);
                                ui.close();
                            }
                            if ui.button("在本卷下新建章节").clicked() {
                                new_chapter_in = Some(vol.id.clone());
                                ui.close();
                            }
                            if ui.button(RichText::new("删除").color(pal.danger)).clicked() {
                                app.dialog = Some(DialogKind::DeleteChapter(c.id.clone(), c.title.clone()));
                                ui.close();
                            }
                        });
                    }
                    if vol.chapters.is_empty() {
                        ui.label(RichText::new("（空卷）").size(11.5).color(pal.text_disabled));
                    }
                });
            // 卷操作
            let (rect, resp) = ui.allocate_exact_size(
                Vec2::new(ui.available_width() - 20.0, 18.0),
                Sense::click(),
            );
            let _ = rect;
            if resp.hovered() {
                ui.painter().text(
                    egui::pos2(ui.cursor().min.x + 6.0, ui.cursor().min.y + 9.0),
                    Align2::LEFT_CENTER,
                    "⋯ 卷操作",
                    FontId::proportional(10.5),
                    pal.text_disabled,
                );
            }
            resp.context_menu(|ui| {
                if ui.button("重命名卷").clicked() {
                    rename_vol = Some(vol.id.clone());
                    ui.close();
                }
                if ui.button("在本卷下新建章节").clicked() {
                    new_chapter_in = Some(vol.id.clone());
                    ui.close();
                }
                if ui.button(RichText::new("删除卷（含章节）").color(pal.danger)).clicked() {
                    delete_vol = Some((vol.id.clone(), vol.title.clone()));
                    ui.close();
                }
            });
            ui.add_space(4.0);
        }
        if volumes.is_empty() {
            ui.label(RichText::new("（暂无章节，右键或点上方 + 新建）").size(11.5).color(pal.text_disabled));
        }
    });

    if let Some(id) = rename_vol {
        app.dialog = Some(DialogKind::RenameVolume(id));
    }
    if let Some((id, t)) = delete_vol {
        app.dialog = Some(DialogKind::DeleteVolume(id, t));
    }
    if let Some(vid) = new_chapter_in {
        app.form_title.clear();
        app.dialog = Some(DialogKind::NewChapter);
        app.sel_chain = Some(vid);
    }
}

fn move_chapter(app: &mut AppState, cid: &str, delta: i32) {
    let Some(novel) = app.novel.as_mut() else { return };
    for v in &mut novel.volumes {
        let Some(pos) = v.chapters.iter().position(|c| c.id == cid) else { continue };
        let new_pos = pos as i32 + delta;
        if new_pos >= 0 && (new_pos as usize) < v.chapters.len() {
            let c = v.chapters.remove(pos);
            v.chapters.insert(new_pos as usize, c);
            app.meta_dirty = true;
        }
    }
}

// ---------- 大纲面板 ----------
fn outline_node_ui(ui: &mut egui::Ui, app: &mut AppState, node: &OutlineNode, depth: usize) {
    let pal = app.pal;
    let selected = app.sel_outline.as_deref() == Some(node.id.as_str());
    let children = node.children.clone();
    let kind_color = match node.kind.as_str() {
        "卷" => pal.accent,
        "章" => pal.text,
        _ => pal.text_secondary,
    };

    let header_text = RichText::new(format!(
        "{}{} {}",
        "  ".repeat(depth.min(4)),
        if node.kind == "卷" { "📘" } else if node.kind == "章" { "📄" } else { "·" },
        node.title
    ))
    .size(12.5)
    .color(if selected { pal.accent } else { kind_color });

    let resp = egui::CollapsingHeader::new(header_text)
        .id_salt(("outline", &node.id))
        .default_open(depth < 2)
        .show(ui, |ui| {
            for child in &children {
                outline_node_ui(ui, app, child, depth + 1);
            }
            if children.is_empty() && !node.content.is_empty() {
                ui.label(
                    RichText::new(format!("    {}", crate::util::truncate_chars(&node.content, 30)))
                        .size(11.0)
                        .color(pal.text_disabled),
                );
            }
        });

    if resp.header_response.clicked() {
        app.sel_outline = Some(node.id.clone());
    }
    resp.header_response.context_menu(|ui| {
        if ui.button("选中").clicked() {
            app.sel_outline = Some(node.id.clone());
            ui.close();
        }
        for kind in ["卷", "章", "节", "要点"] {
            let label = format!("添加子{}", kind);
            if ui.button(label).clicked() {
                app.new_name.clear();
                app.sel_outline = Some(node.id.clone());
                app.sel_chain = Some(format!("new:{}:{}", kind, node.id));
                ui.close();
            }
        }
        if ui.button("上移").clicked() {
            move_outline(app, &node.id, -1);
            ui.close();
        }
        if ui.button("下移").clicked() {
            move_outline(app, &node.id, 1);
            ui.close();
        }
        if ui.button("删除").clicked() {
            delete_outline_node(app, &node.id);
            ui.close();
        }
    });
}

pub fn outline_panel(ui: &mut egui::Ui, app: &mut AppState) {
    let pal = app.pal;
    let Some(novel) = app.novel.as_ref() else { return };
    let _outline = novel.outline.clone();

    // 新建节点请求（来自右键菜单）
    if let Some(req) = app.sel_chain.clone() {
        if let Some(rest) = req.strip_prefix("new:") {
            let (kind, parent) = rest.split_once(':').unwrap_or(("节", ""));
            app.sel_chain = None;
            let mut new_node = OutlineNode {
                id: crate::util::new_id(),
                title: "新节点".into(),
                kind: kind.to_string(),
                content: String::new(),
                children: Vec::new(),
            };
            if parent.is_empty() {
                if let Some(n) = app.novel.as_mut() {
                    n.outline.push(new_node.clone());
                    app.sel_outline = Some(new_node.id.clone());
                    new_node = OutlineNode::default();
                }
            } else if let Some(n) = app.novel.as_mut() {
                if let Some(node) = find_outline_mut(&mut n.outline, parent) {
                    node.children.push(new_node.clone());
                    app.sel_outline = Some(new_node.id.clone());
                    new_node = OutlineNode::default();
                }
            }
            app.meta_dirty = true;
        }
    }

    // AI 生成按钮
    ui.horizontal(|ui| {
        if widgets::secondary_btn(ui, "✨ 生成大纲", &pal).clicked() {
            if !app.ai_panel_open {
                app.ai_panel_open = true;
            }
            let msgs = crate::ai_panel::build_outline_msgs(app);
            app.start_ai("大纲生成", msgs);
        }
        if widgets::secondary_btn(ui, "＋ 添加卷", &pal).clicked() {
            app.form_title.clear();
            app.dialog = Some(DialogKind::NewVolume);
        }
    });
    ui.add_space(4.0);
    widgets::h_sep(ui, &pal);
    ui.add_space(4.0);

    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        let outline = app.novel.as_ref().map(|n| n.outline.clone()).unwrap_or_default();
        for node in &outline {
            outline_node_ui(ui, app, node, 0);
        }
        if outline.is_empty() {
            ui.label(RichText::new("（暂无大纲，点击「生成大纲」或右键添加节点）").size(11.5).color(pal.text_disabled));
        }
    });
}

fn find_outline_mut<'a>(nodes: &'a mut Vec<OutlineNode>, id: &str) -> Option<&'a mut OutlineNode> {
    for n in nodes.iter_mut() {
        if n.id == id {
            return Some(n);
        }
        if let Some(found) = find_outline_mut(&mut n.children, id) {
            return Some(found);
        }
    }
    None
}

fn delete_outline_node(app: &mut AppState, id: &str) {
    fn rec(nodes: &mut Vec<OutlineNode>, id: &str) -> bool {
        if let Some(pos) = nodes.iter().position(|n| n.id == id) {
            nodes.remove(pos);
            return true;
        }
        for n in nodes.iter_mut() {
            if rec(&mut n.children, id) {
                return true;
            }
        }
        false
    }
    if let Some(novel) = app.novel.as_mut() {
        if rec(&mut novel.outline, id) {
            app.sel_outline = None;
            app.meta_dirty = true;
        }
    }
}

fn move_outline(app: &mut AppState, id: &str, delta: i32) {
    fn rec(nodes: &mut Vec<OutlineNode>, id: &str, delta: i32) -> bool {
        if let Some(pos) = nodes.iter().position(|n| n.id == id) {
            let new_pos = pos as i32 + delta;
            if new_pos >= 0 && (new_pos as usize) < nodes.len() {
                let n = nodes.remove(pos);
                nodes.insert(new_pos as usize, n);
                return true;
            }
        }
        for n in nodes.iter_mut() {
            if rec(&mut n.children, id, delta) {
                return true;
            }
        }
        false
    }
    if let Some(novel) = app.novel.as_mut() {
        if rec(&mut novel.outline, id, delta) {
            app.meta_dirty = true;
        }
    }
}

/// 大纲节点详情编辑（中央区）
pub fn outline_detail(ui: &mut egui::Ui, app: &mut AppState) {
    let pal = app.pal;
    let Some(id) = app.sel_outline.clone() else {
        widgets::empty_state(ui, "🗂", "在左侧选中一个大纲节点", "可在此编辑标题与内容要点", &pal);
        return;
    };
    let Some(node) = app.novel.as_ref().and_then(|n| find_outline(&n.outline, &id)).cloned() else {
        return;
    };
    ui.add_space(12.0);
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui.label(RichText::new("🗂 大纲节点").size(16.0).strong().color(pal.text));
        ui.label(
            RichText::new(format!("类型：{}", node.kind))
                .size(12.0)
                .color(pal.accent),
        );
    });
    ui.add_space(8.0);
    let mut title = node.title.clone();
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui.label(RichText::new("标题").size(12.5).color(pal.text_secondary));
        if widgets::text_input(ui, &mut title, "节点标题", &pal).changed() {
            if let Some(n) = app.novel.as_mut().and_then(|n| find_outline_mut(&mut n.outline, &id)) {
                n.title = title.trim().to_string();
                app.meta_dirty = true;
            }
        }
    });
    ui.add_space(6.0);
    let mut content = node.content.clone();
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui.label(RichText::new("内容").size(12.5).color(pal.text_secondary));
    });
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        if widgets::text_area(ui, &mut content, "剧情要点 / 备注…", 8, &pal).changed() {
            if let Some(n) = app.novel.as_mut().and_then(|n| find_outline_mut(&mut n.outline, &id)) {
                n.content = content.clone();
                app.meta_dirty = true;
            }
        }
    });
}

fn find_outline<'a>(nodes: &'a [OutlineNode], id: &str) -> Option<&'a OutlineNode> {
    for n in nodes {
        if n.id == id {
            return Some(n);
        }
        if let Some(found) = find_outline(&n.children, id) {
            return Some(found);
        }
    }
    None
}
