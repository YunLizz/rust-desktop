//! 人物 / 世界观 / 时间线 / 任务 视图

use eframe::egui::{self, Align, Align2, CornerRadius, FontId, Layout, Rect, RichText, Sense, Stroke, Vec2};

use crate::app::AppState;
use crate::model::{Location, TimelineEvent, Task, TaskChain};
use crate::util;
use crate::widgets;

// ================= 人物 =================
pub fn characters_panel(ui: &mut egui::Ui, app: &mut AppState) {
    let pal = app.pal;
    widgets::section_header(ui, "人物卡", &pal);
    ui.horizontal(|ui| {
        if widgets::secondary_btn(ui, "＋ 新建人物", &pal).clicked() {
            app.new_name.clear();
            app.sel_char = None;
            app.dialog = Some(crate::app::DialogKind::NewChapter); // 占位，不触发
            app.dialog = None;
            if let Some(n) = app.novel.as_mut() {
                let c = crate::model::Character {
                    id: util::new_id(),
                    name: "新人物".into(),
                    ..Default::default()
                };
                n.characters.push(c.clone());
                app.sel_char = Some(c.id);
                app.meta_dirty = true;
            }
        }
        if widgets::secondary_btn(ui, "🧩 关系网", &pal).clicked() {
            app.show_relation_canvas = !app.show_relation_canvas;
        }
    });
    ui.add_space(4.0);
    widgets::h_sep(ui, &pal);
    ui.add_space(4.0);

    let Some(novel) = app.novel.as_ref() else { return };
    let chars = novel.characters.clone();
    let mut open_canvas = false;
    if app.show_relation_canvas {
        open_canvas = true;
    }
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        for c in &chars {
            let selected = app.sel_char.as_deref() == Some(c.id.as_str());
            let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 30.0), Sense::click());
            if selected {
                ui.painter().rect_filled(rect, CornerRadius::same(5), pal.accent.gamma_multiply(0.14));
            } else if resp.hovered() {
                ui.painter().rect_filled(rect, CornerRadius::same(5), pal.bg_hover);
            }
            ui.painter().text(
                egui::pos2(rect.left() + 8.0, rect.center().y),
                Align2::LEFT_CENTER,
                &c.name,
                FontId::proportional(13.0),
                if selected { pal.text } else { pal.text_secondary },
            );
            if !c.role.is_empty() {
                let role_color = match c.role.as_str() {
                    "主角" => pal.accent,
                    "反派" => pal.danger,
                    "重要配角" => pal.warn,
                    _ => pal.text_disabled,
                };
                ui.painter().text(
                    egui::pos2(rect.right() - 8.0, rect.center().y),
                    Align2::RIGHT_CENTER,
                    &c.role,
                    FontId::proportional(10.5),
                    role_color,
                );
            }
            if resp.clicked() {
                app.sel_char = Some(c.id.clone());
            }
            resp.context_menu(|ui| {
                if ui.button("重命名").clicked() {
                    app.new_name = c.name.clone();
                    ui.close();
                }
                if ui.button(RichText::new("删除").color(pal.danger)).clicked() {
                    if let Some(n) = app.novel.as_mut() {
                        n.characters.retain(|x| x.id != c.id);
                        if app.sel_char.as_deref() == Some(c.id.as_str()) {
                            app.sel_char = None;
                        }
                        app.meta_dirty = true;
                    }
                    ui.close();
                }
            });
        }
        if chars.is_empty() {
            ui.label(RichText::new("（暂无人物）").size(11.5).color(pal.text_disabled));
        }
    });
    if open_canvas && app.novel.is_some() {
        // 关系网在中央区展示
    }
}

/// 人物详情（中央区）
pub fn character_detail(ui: &mut egui::Ui, app: &mut AppState) {
    let pal = app.pal;
    let Some(id) = app.sel_char.clone() else {
        widgets::empty_state(ui, "👥", "在左侧选择一个人物", "编辑人物卡，或点「关系网」查看人物关系", &pal);
        return;
    };
    if app.show_relation_canvas {
        relation_canvas(ui, app);
        return;
    }
    let Some(c) = app.novel.as_ref().and_then(|n| n.find_character(&id)).cloned() else {
        return;
    };
    let mut name = c.name.clone();
    let mut role = c.role.clone();
    let mut appearance = c.appearance.clone();
    let mut personality = c.personality.clone();
    let mut background = c.background.clone();
    let mut goals = c.goals.clone();
    let mut notes = c.notes.clone();

    ui.add_space(12.0);
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui.label(RichText::new("👤 人物卡").size(16.0).strong().color(pal.text));
        ui.label(RichText::new("（所有修改自动保存）").size(11.0).color(pal.text_disabled));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add_space(18.0);
            if widgets::secondary_btn(ui, "🧩 关系网", &pal).clicked() {
                app.show_relation_canvas = !app.show_relation_canvas;
            }
            if widgets::secondary_btn(ui, "✨ AI 完善人设", &pal).clicked() {
                let msgs = crate::ai_panel::build_character_msgs(app, &c.name, &c.role);
                app.start_ai("人物卡", msgs);
            }
            if widgets::secondary_btn(ui, "🗑 删除", &pal).clicked() {
                if let Some(n) = app.novel.as_mut() {
                    n.characters.retain(|x| x.id != id);
                    app.sel_char = None;
                    app.meta_dirty = true;
                }
            }
        });
    });
    ui.add_space(10.0);

    let field = |ui: &mut egui::Ui, label: &str, value: &mut String, rows: usize| {
        ui.horizontal(|ui| {
            ui.add_space(18.0);
            ui.label(RichText::new(label).size(12.5).color(pal.text_secondary));
        });
        ui.horizontal(|ui| {
            ui.add_space(18.0);
            if rows == 1 {
                widgets::text_input(ui, value, "", &pal);
            } else {
                widgets::text_area(ui, value, "", rows, &pal);
            }
        });
        ui.add_space(4.0);
    };

    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui.label(RichText::new("名字").size(12.5).color(pal.text_secondary));
        if widgets::text_input(ui, &mut name, "人物名字", &pal).changed() {
            if let Some(c) = app.novel.as_mut().and_then(|n| n.find_character_mut(&id)) {
                c.name = name.trim().to_string();
                app.meta_dirty = true;
            }
        }
        ui.add_space(20.0);
        ui.label(RichText::new("定位").size(12.5).color(pal.text_secondary));
        egui::ComboBox::from_id_salt(("role", &id))
            .selected_text(if role.is_empty() { "选择…" } else { role.as_str() })
            .width(110.0)
            .show_ui(ui, |ui| {
                for r in ["主角", "重要配角", "配角", "反派", "其他"] {
                    if ui.selectable_label(role == r, r).clicked() {
                        role = r.to_string();
                        if let Some(c) = app.novel.as_mut().and_then(|n| n.find_character_mut(&id)) {
                            c.role = role.clone();
                            app.meta_dirty = true;
                        }
                    }
                }
            });
    });
    ui.add_space(6.0);
    field(ui, "外貌", &mut appearance, 2);
    field(ui, "性格", &mut personality, 2);
    field(ui, "背景经历", &mut background, 3);
    field(ui, "目标与欲望", &mut goals, 2);
    field(ui, "备注", &mut notes, 2);

    // 关系
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui.label(RichText::new("关系").size(12.5).color(pal.text_secondary));
        ui.label(RichText::new("（如：妹妹、宿敌、导师…）").size(11.0).color(pal.text_disabled));
    });
    let chars = app
        .novel
        .as_ref()
        .map(|n| {
            n.characters
                .iter()
                .filter(|x| x.id != id)
                .map(|x| (x.id.clone(), x.name.clone()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut target = app.new_name.clone();
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        egui::ComboBox::from_id_salt(("rel_target", &id))
            .selected_text("选择对象…")
            .width(110.0)
            .show_ui(ui, |ui| {
                for (_cid, cname) in &chars {
                    if ui.selectable_label(false, cname).clicked() {
                        target = cname.clone();
                    }
                }
            });
        let _ = target;
    });
    // 简化：直接以列表方式管理关系
    let rels = c.relationships.clone();
    for (i, rel) in rels.iter().enumerate() {
        ui.horizontal(|ui| {
            ui.add_space(22.0);
            ui.label(RichText::new(format!("↔ {}", rel.target_name)).size(12.5).color(pal.accent));
            ui.label(RichText::new(&rel.relation).size(12.0).color(pal.text_secondary));
            if widgets::icon_btn(ui, "✕", "删除关系", &pal).clicked() {
                if let Some(c) = app.novel.as_mut().and_then(|n| n.find_character_mut(&id)) {
                    c.relationships.remove(i);
                    app.meta_dirty = true;
                }
            }
        });
    }
    // 添加关系
    let mut rel_target = app.sel_chain.clone().unwrap_or_default();
    let mut rel_text = app.new_name.clone();
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui.label(RichText::new("添加关系：").size(12.0).color(pal.text_secondary));
    });
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        egui::ComboBox::from_id_salt("rel_add")
            .selected_text(if rel_target.is_empty() { "选择对象…" } else { rel_target.as_str() })
            .width(120.0)
            .show_ui(ui, |ui| {
                for (cid, cname) in &chars {
                    if ui.selectable_label(rel_target == *cid, cname).clicked() {
                        rel_target = cid.clone();
                    }
                }
            });
        if widgets::text_input(ui, &mut rel_text, "关系（如：兄妹）", &pal).changed() {
            app.new_name = rel_text.clone();
        }
        if widgets::secondary_btn(ui, "添加", &pal).clicked() && !rel_target.is_empty() {
            if let Some(c) = app.novel.as_mut().and_then(|n| n.find_character_mut(&id)) {
                let tname = chars
                    .iter()
                    .find(|(cid, _)| cid == &rel_target)
                    .map(|(_, n)| n.clone())
                    .unwrap_or_default();
                c.relationships.push(crate::model::Relationship {
                    target_id: rel_target.clone(),
                    target_name: tname,
                    relation: rel_text.trim().to_string(),
                    note: String::new(),
                });
                app.meta_dirty = true;
                app.new_name.clear();
                app.sel_chain = None;
            }
        }
    });
    ui.add_space(8.0);
}

/// 关系网画布
fn relation_canvas(ui: &mut egui::Ui, app: &mut AppState) {
    let pal = app.pal;
    let Some(novel) = app.novel.as_ref() else { return };
    let chars = novel.characters.clone();
    let canvas_size = Vec2::new(1800.0, 1200.0);

    // 初始化节点位置（环形）
    let mut need_save = false;
    if app.canvas_pos.is_empty() && !chars.is_empty() {
        let n = chars.len();
        for (i, c) in chars.iter().enumerate() {
            let angle = std::f32::consts::TAU * i as f32 / n as f32;
            let r = 300.0;
            app.canvas_pos.insert(
                c.id.clone(),
                egui::pos2(canvas_size.x / 2.0 + r * angle.cos(), canvas_size.y / 2.0 + r * angle.sin()),
            );
        }
    }

    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui.label(RichText::new("🧩 人物关系网").size(16.0).strong().color(pal.text));
        ui.label(RichText::new("（拖动节点调整布局）").size(11.5).color(pal.text_disabled));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add_space(18.0);
            if widgets::secondary_btn(ui, "返回人物卡", &pal).clicked() {
                app.show_relation_canvas = false;
            }
            if widgets::secondary_btn(ui, "重置布局", &pal).clicked() {
                app.canvas_pos.clear();
            }
        });
    });
    ui.add_space(6.0);

    egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
        let (rect, _) = ui.allocate_exact_size(canvas_size, Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, pal.bg_panel);

        // 连线
        for c in &chars {
            for rel in &c.relationships {
                let (Some(p1), Some(p2)) = (app.canvas_pos.get(&c.id), app.canvas_pos.get(&rel.target_id)) else {
                    continue;
                };
                painter.line_segment([*p1, *p2], Stroke::new(1.5, pal.border));
                let mid = egui::pos2((p1.x + p2.x) / 2.0, (p1.y + p2.y) / 2.0);
                if !rel.relation.is_empty() {
                    painter.text(
                        mid,
                        Align2::CENTER_CENTER,
                        &rel.relation,
                        FontId::proportional(11.0),
                        pal.text_disabled,
                    );
                }
            }
        }
        // 节点
        for c in &chars {
            let Some(pos) = app.canvas_pos.get(&c.id).copied() else { continue };
            let node = Rect::from_center_size(pos, Vec2::new(120.0, 44.0));
            let resp = ui.interact(node, ui.id().with(("cnode", &c.id)), Sense::click_and_drag());
            if resp.dragged() {
                if let Some(ptr) = ui.ctx().pointer_interact_pos() {
                    app.canvas_pos.insert(c.id.clone(), ptr - Vec2::new(0.0, 0.0));
                    need_save = true;
                }
            }
            let bg = if app.sel_char.as_deref() == Some(c.id.as_str()) {
                pal.accent.gamma_multiply(0.3)
            } else if resp.hovered() {
                pal.bg_hover
            } else {
                pal.bg_panel_alt
            };
            painter.rect_filled(node, CornerRadius::same(10), bg);
            painter.rect_stroke(node, CornerRadius::same(10), Stroke::new(1.0, pal.accent.gamma_multiply(0.5)), egui::StrokeKind::Middle);
            painter.text(
                node.center(),
                Align2::CENTER_CENTER,
                &c.name,
                FontId::proportional(13.5),
                pal.text,
            );
            if resp.clicked() {
                app.sel_char = Some(c.id.clone());
            }
        }
        if chars.is_empty() {
            painter.text(
                rect.center(),
                Align2::CENTER_CENTER,
                "（暂无人物）",
                FontId::proportional(14.0),
                pal.text_disabled,
            );
        }
    });
    if need_save {
        app.meta_dirty = false; // 画布位置不落盘
    }
}

// ================= 世界观 =================
pub fn world_panel(ui: &mut egui::Ui, app: &mut AppState) {
    let pal = app.pal;
    widgets::section_header(ui, "世界观 / 地点", &pal);
    ui.horizontal(|ui| {
        if widgets::secondary_btn(ui, "＋ 新建地点", &pal).clicked() {
            if let Some(n) = app.novel.as_mut() {
                let loc = Location {
                    id: util::new_id(),
                    name: "新地点".into(),
                    kind: "城市".into(),
                    parent_id: None,
                    description: String::new(),
                };
                n.locations.push(loc.clone());
                app.sel_loc = Some(loc.id);
                app.meta_dirty = true;
            }
        }
        if widgets::secondary_btn(ui, "✨ AI 生成设定", &pal).clicked() {
            let msgs = crate::ai_panel::build_world_msgs(app, "", "");
            app.start_ai("世界观", msgs);
        }
    });
    ui.add_space(4.0);
    widgets::h_sep(ui, &pal);
    ui.add_space(4.0);

    let Some(novel) = app.novel.as_ref() else { return };
    let locs = novel.locations.clone();
    let mut root: Vec<&crate::model::Location> = locs.iter().filter(|l| l.parent_id.is_none()).collect();
    root.sort_by(|a, b| a.name.cmp(&b.name));
    // 有父节点的按父分组递归
    fn draw_tree(ui: &mut egui::Ui, app: &mut AppState, parents: &[&Location], locs: &[Location], depth: usize) {
        let pal = app.pal;
        for loc in parents {
            let selected = app.sel_loc.as_deref() == Some(loc.id.as_str());
            let children: Vec<&Location> = locs.iter().filter(|l| l.parent_id.as_deref() == Some(loc.id.as_str())).collect();
            let label = RichText::new(format!(
                "{}{} {}",
                "  ".repeat(depth.min(3)),
                loc_kind_icon(&loc.kind),
                loc.name
            ))
            .size(12.5)
            .color(if selected { pal.accent } else { pal.text });
            let resp = egui::CollapsingHeader::new(label)
                .id_salt(("loc", &loc.id))
                .default_open(true)
                .show(ui, |ui| {
                    draw_tree(ui, app, &children, locs, depth + 1);
                });
            if resp.header_response.clicked() {
                app.sel_loc = Some(loc.id.clone());
            }
            resp.header_response.context_menu(|ui| {
                if ui.button("重命名").clicked() {
                    app.new_name = loc.name.clone();
                    ui.close();
                }
                if ui.button(RichText::new("删除").color(pal.danger)).clicked() {
                    if let Some(n) = app.novel.as_mut() {
                        n.locations.retain(|x| x.id != loc.id && x.parent_id.as_deref() != Some(loc.id.as_str()));
                        if app.sel_loc.as_deref() == Some(loc.id.as_str()) {
                            app.sel_loc = None;
                        }
                        app.meta_dirty = true;
                    }
                    ui.close();
                }
            });
        }
    }
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        draw_tree(ui, app, &root, &locs, 0);
        if locs.is_empty() {
            ui.label(RichText::new("（暂无地点设定）").size(11.5).color(pal.text_disabled));
        }
    });
}

fn loc_kind_icon(kind: &str) -> &str {
    match kind {
        "国家" => "🏳️",
        "城市" => "🏙️",
        "地区" => "🏞️",
        "建筑" => "🏛️",
        "异界" => "🌌",
        _ => "📍",
    }
}

/// 地点详情（中央区）
pub fn location_detail(ui: &mut egui::Ui, app: &mut AppState) {
    let pal = app.pal;
    let Some(id) = app.sel_loc.clone() else {
        widgets::empty_state(ui, "🗺", "在左侧选择一个地点", "管理世界观设定", &pal);
        return;
    };
    let Some(loc) = app.novel.as_ref().and_then(|n| n.find_location(&id)).cloned() else {
        return;
    };
    let mut name = loc.name.clone();
    let mut kind = loc.kind.clone();
    let mut desc = loc.description.clone();

    ui.add_space(12.0);
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui.label(RichText::new("🗺 地点设定").size(16.0).strong().color(pal.text));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add_space(18.0);
            if widgets::secondary_btn(ui, "🗑 删除", &pal).clicked() {
                if let Some(n) = app.novel.as_mut() {
                    n.locations.retain(|x| x.id != id);
                    app.sel_loc = None;
                    app.meta_dirty = true;
                }
            }
        });
    });
    ui.add_space(10.0);
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui.label(RichText::new("名称").size(12.5).color(pal.text_secondary));
        if widgets::text_input(ui, &mut name, "地点名称", &pal).changed() {
            if let Some(l) = app.novel.as_mut().and_then(|n| n.find_location_mut(&id)) {
                l.name = name.trim().to_string();
                app.meta_dirty = true;
            }
        }
        ui.add_space(20.0);
        ui.label(RichText::new("类别").size(12.5).color(pal.text_secondary));
        egui::ComboBox::from_id_salt(("loc_kind", &id))
            .selected_text(kind.as_str())
            .width(100.0)
            .show_ui(ui, |ui| {
                for k in ["国家", "城市", "地区", "建筑", "异界", "其他"] {
                    if ui.selectable_label(kind == k, k).clicked() {
                        kind = k.to_string();
                        if let Some(l) = app.novel.as_mut().and_then(|n| n.find_location_mut(&id)) {
                            l.kind = kind.clone();
                            app.meta_dirty = true;
                        }
                    }
                }
            });
    });
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui.label(RichText::new("描述").size(12.5).color(pal.text_secondary));
    });
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        if widgets::text_area(ui, &mut desc, "地点外貌、氛围、人文…", 10, &pal).changed() {
            if let Some(l) = app.novel.as_mut().and_then(|n| n.find_location_mut(&id)) {
                l.description = desc.clone();
                app.meta_dirty = true;
            }
        }
    });
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui.label(RichText::new("子地点").size(12.5).color(pal.text_secondary));
        let children: Vec<String> = app
            .novel
            .as_ref()
            .map(|n| {
                n.locations
                    .iter()
                    .filter(|l| l.parent_id.as_deref() == Some(id.as_str()))
                    .map(|l| l.name.clone())
                    .collect()
            })
            .unwrap_or_default();
        if children.is_empty() {
            ui.label(RichText::new("（无）").size(12.0).color(pal.text_disabled));
        } else {
            ui.label(RichText::new(children.join("、")).size(12.0).color(pal.text_secondary));
        }
    });
}

// ================= 时间线 =================
pub fn timeline_panel(ui: &mut egui::Ui, app: &mut AppState) {
    let pal = app.pal;
    widgets::section_header(ui, "时间线", &pal);
    if widgets::secondary_btn(ui, "＋ 新建事件", &pal).clicked() {
        if let Some(n) = app.novel.as_mut() {
            let e = TimelineEvent {
                id: util::new_id(),
                title: "新事件".into(),
                time: "第1卷".into(),
                description: String::new(),
                character_ids: Vec::new(),
                location_id: None,
                chapter_id: None,
            };
            n.timeline.push(e.clone());
            app.sel_event = Some(e.id);
            app.meta_dirty = true;
        }
    }
    ui.add_space(4.0);
    widgets::h_sep(ui, &pal);
    ui.add_space(4.0);

    let Some(novel) = app.novel.as_ref() else { return };
    let events = novel.timeline.clone();
    let mut sorted = events.clone();
    sorted.sort_by(|a, b| a.time.cmp(&b.time));
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        for e in &sorted {
            let selected = app.sel_event.as_deref() == Some(e.id.as_str());
            let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 30.0), Sense::click());
            if selected {
                ui.painter().rect_filled(rect, CornerRadius::same(5), pal.accent.gamma_multiply(0.14));
            } else if resp.hovered() {
                ui.painter().rect_filled(rect, CornerRadius::same(5), pal.bg_hover);
            }
            ui.painter().text(
                egui::pos2(rect.left() + 8.0, rect.center().y),
                Align2::LEFT_CENTER,
                format!("⏱ {}", e.title),
                FontId::proportional(12.5),
                if selected { pal.text } else { pal.text_secondary },
            );
            ui.painter().text(
                egui::pos2(rect.right() - 8.0, rect.center().y),
                Align2::RIGHT_CENTER,
                &e.time,
                FontId::proportional(10.5),
                pal.text_disabled,
            );
            if resp.clicked() {
                app.sel_event = Some(e.id.clone());
            }
            resp.context_menu(|ui| {
                if ui.button(RichText::new("删除").color(pal.danger)).clicked() {
                    if let Some(n) = app.novel.as_mut() {
                        n.timeline.retain(|x| x.id != e.id);
                        if app.sel_event.as_deref() == Some(e.id.as_str()) {
                            app.sel_event = None;
                        }
                        app.meta_dirty = true;
                    }
                    ui.close();
                }
            });
        }
        if events.is_empty() {
            ui.label(RichText::new("（暂无事件）").size(11.5).color(pal.text_disabled));
        }
    });
}

pub fn timeline_detail(ui: &mut egui::Ui, app: &mut AppState) {
    let pal = app.pal;
    let Some(id) = app.sel_event.clone() else {
        widgets::empty_state(ui, "⏱", "在左侧选择或新建一个事件", "按时间排序梳理剧情脉络", &pal);
        return;
    };
    let Some(e) = app.novel.as_ref().and_then(|n| n.timeline.iter().find(|x| x.id == id)).cloned() else {
        return;
    };
    let mut title = e.title.clone();
    let mut time = e.time.clone();
    let mut desc = e.description.clone();

    ui.add_space(12.0);
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui.label(RichText::new("⏱ 事件").size(16.0).strong().color(pal.text));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add_space(18.0);
            if widgets::secondary_btn(ui, "🗑 删除", &pal).clicked() {
                if let Some(n) = app.novel.as_mut() {
                    n.timeline.retain(|x| x.id != id);
                    app.sel_event = None;
                    app.meta_dirty = true;
                }
            }
        });
    });
    ui.add_space(10.0);
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui.label(RichText::new("标题").size(12.5).color(pal.text_secondary));
        if widgets::text_input(ui, &mut title, "事件标题", &pal).changed() {
            if let Some(x) = app.novel.as_mut().and_then(|n| n.timeline.iter_mut().find(|x| x.id == id)) {
                x.title = title.trim().to_string();
                app.meta_dirty = true;
            }
        }
    });
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui.label(RichText::new("时间").size(12.5).color(pal.text_secondary));
        if widgets::text_input(ui, &mut time, "如：第一卷 第3章 前夜", &pal).changed() {
            if let Some(x) = app.novel.as_mut().and_then(|n| n.timeline.iter_mut().find(|x| x.id == id)) {
                x.time = time.trim().to_string();
                app.meta_dirty = true;
            }
        }
    });
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui.label(RichText::new("描述").size(12.5).color(pal.text_secondary));
    });
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        if widgets::text_area(ui, &mut desc, "发生了什么…", 8, &pal).changed() {
            if let Some(x) = app.novel.as_mut().and_then(|n| n.timeline.iter_mut().find(|x| x.id == id)) {
                x.description = desc.clone();
                app.meta_dirty = true;
            }
        }
    });
    ui.add_space(6.0);
    // 关联章节
    let chapters: Vec<(String, String)> = app
        .novel
        .as_ref()
        .map(|n| {
            n.chapters_all()
                .iter()
                .map(|c| (c.id.clone(), c.title.clone()))
                .collect()
        })
        .unwrap_or_default();
    let mut linked = e.chapter_id.clone().unwrap_or_default();
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui.label(RichText::new("关联章节").size(12.5).color(pal.text_secondary));
        egui::ComboBox::from_id_salt(("ev_ch", &id))
            .selected_text(if linked.is_empty() { "无" } else { "已关联" })
            .width(100.0)
            .show_ui(ui, |ui| {
                if ui.selectable_label(linked.is_empty(), "无").clicked() {
                    linked.clear();
                }
                for (cid, cname) in &chapters {
                    if ui.selectable_label(linked == *cid, cname).clicked() {
                        linked = cid.clone();
                    }
                }
            });
        if let Some(x) = app.novel.as_mut().and_then(|n| n.timeline.iter_mut().find(|x| x.id == id)) {
            x.chapter_id = if linked.is_empty() { None } else { Some(linked.clone()) };
            app.meta_dirty = true;
        }
    });
}

// ================= 任务 =================
pub fn tasks_panel(ui: &mut egui::Ui, app: &mut AppState) {
    let pal = app.pal;
    widgets::section_header(ui, "任务链", &pal);
    let Some(novel) = app.novel.as_ref() else { return };
    let chains = novel.chains.clone();
    let all_tasks: Vec<Task> = novel.tasks.clone();

    let mut sel = app.sel_chain.clone().unwrap_or_default();
    if sel.is_empty() && !chains.is_empty() {
        sel = "all".into();
    }
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        if ui
            .selectable_label(sel == "all", format!("📋 全部任务（{}）", all_tasks.len()))
            .clicked()
        {
            app.sel_chain = Some("all".into());
        }
        for ch in &chains {
            let cnt = all_tasks.iter().filter(|t| t.chain_id.as_deref() == Some(ch.id.as_str())).count();
            if ui
                .selectable_label(sel == ch.id, format!("🔗 {}（{}）", ch.name, cnt))
                .clicked()
            {
                app.sel_chain = Some(ch.id.clone());
            }
        }
        if chains.is_empty() {
            ui.label(RichText::new("（暂无任务链）").size(11.5).color(pal.text_disabled));
        }
    });
}

/// 任务看板（中央区）
pub fn task_board(ui: &mut egui::Ui, app: &mut AppState) {
    let pal = app.pal;
    ui.add_space(12.0);
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui.label(RichText::new("🎯 任务看板").size(16.0).strong().color(pal.text));
        ui.label(RichText::new("（待办 → 进行中 → 已完成）").size(11.5).color(pal.text_disabled));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add_space(18.0);
            if widgets::secondary_btn(ui, "＋ 新建任务链", &pal).clicked() {
                app.new_name.clear();
                app.dialog = Some(crate::app::DialogKind::NewVolume); // 占位
                app.dialog = None;
                if let Some(n) = app.novel.as_mut() {
                    let chain_id = util::new_id();
                    n.chains.push(TaskChain {
                        id: chain_id.clone(),
                        name: "新任务链".into(),
                        description: String::new(),
                        task_ids: Vec::new(),
                    });
                    app.sel_chain = Some(chain_id);
                    app.meta_dirty = true;
                }
            }
            if widgets::secondary_btn(ui, "＋ 新建任务", &pal).clicked() {
                app.new_name.clear();
                app.sel_chain = app.sel_chain.clone();
                app.dialog = Some(crate::app::DialogKind::NewChapter); // 占位
                app.dialog = None;
                let chain_id = if app.sel_chain.as_deref() == Some("all") { None } else { app.sel_chain.clone() };
                if let Some(n) = app.novel.as_mut() {
                    n.tasks.push(Task {
                        id: util::new_id(),
                        title: "新任务".into(),
                        description: String::new(),
                        status: 0,
                        chain_id: chain_id.clone(),
                    });
                    app.meta_dirty = true;
                }
            }
        });
    });
    ui.add_space(10.0);

    let Some(novel) = app.novel.as_ref() else { return };
    let chain_filter = app.sel_chain.clone();
    let tasks: Vec<Task> = novel
        .tasks
        .iter()
        .filter(|t| match chain_filter.as_deref() {
            Some("all") | None => true,
            Some(cid) => t.chain_id.as_deref() == Some(cid),
        })
        .cloned()
        .collect();

    let cols = ["待办", "进行中", "已完成"];
    let col_colors = [pal.text_secondary, pal.warn, pal.ok];
    let mut move_task: Option<(String, u8)> = None;
    let mut del_task: Option<String> = None;
    let mut cycle: Option<String> = None;

    ui.horizontal(|ui| {
        ui.add_space(18.0);
        for (ci, cname) in cols.iter().enumerate() {
            let col_tasks: Vec<&Task> = tasks.iter().filter(|t| t.status as usize == ci).collect();
            let col_w = 220.0;
            let h = ui.available_height().max(200.0);
            let (rect, _) = ui.allocate_exact_size(Vec2::new(col_w, h), Sense::hover());
            ui.painter().rect_filled(rect, CornerRadius::same(10), pal.bg_panel);
            ui.painter().rect_stroke(rect, CornerRadius::same(10), Stroke::new(1.0, pal.border), egui::StrokeKind::Middle);
            // 列头
            ui.painter().rect_filled(
                Rect::from_min_size(rect.min, Vec2::new(col_w, 34.0)),
                CornerRadius::same(10),
                col_colors[ci].gamma_multiply(0.15),
            );
            ui.painter().text(
                egui::pos2(rect.left() + 12.0, rect.top() + 17.0),
                Align2::LEFT_CENTER,
                format!("{}（{}）", cname, col_tasks.len()),
                FontId::proportional(13.0),
                col_colors[ci],
            );
            // 卡片
            let mut y = rect.top() + 42.0;
            for t in col_tasks {
                let card = Rect::from_min_size(
                    egui::pos2(rect.left() + 8.0, y),
                    Vec2::new(col_w - 16.0, 58.0),
                );
                let resp = ui.interact(card, ui.id().with(("task", t.id.clone())), Sense::click());
                ui.painter().rect_filled(card, CornerRadius::same(8), pal.bg_panel_alt);
                ui.painter().rect_stroke(card, CornerRadius::same(8), Stroke::new(1.0, pal.border), egui::StrokeKind::Middle);
                ui.painter().text(
                    egui::pos2(card.left() + 10.0, card.top() + 13.0),
                    Align2::LEFT_CENTER,
                    &t.title,
                    FontId::proportional(12.5),
                    pal.text,
                );
                if !t.description.is_empty() {
                    ui.painter().text(
                        egui::pos2(card.left() + 10.0, card.top() + 33.0),
                        Align2::LEFT_CENTER,
                        util::truncate_chars(&t.description, 18),
                        FontId::proportional(10.5),
                        pal.text_disabled,
                    );
                }
                // 操作按钮
                if ci > 0 {
                    if ui
                        .interact(
                            Rect::from_min_size(egui::pos2(card.left() + 4.0, card.bottom() - 18.0), Vec2::new(18.0, 16.0)),
                            ui.id().with(("task_left", t.id.clone())),
                            Sense::click(),
                        )
                        .clicked()
                    {
                        move_task = Some((t.id.clone(), ci as u8 - 1));
                    }
                }
                if ci < 2 {
                    if ui
                        .interact(
                            Rect::from_min_size(egui::pos2(card.right() - 22.0, card.bottom() - 18.0), Vec2::new(18.0, 16.0)),
                            ui.id().with(("task_right", t.id.clone())),
                            Sense::click(),
                        )
                        .clicked()
                    {
                        move_task = Some((t.id.clone(), ci as u8 + 1));
                    }
                }
                ui.painter().text(
                    egui::pos2(card.left() + 10.0, card.bottom() - 10.0),
                    Align2::LEFT_CENTER,
                    "◀",
                    FontId::proportional(9.0),
                    pal.text_disabled,
                );
                ui.painter().text(
                    egui::pos2(card.right() - 16.0, card.bottom() - 10.0),
                    Align2::LEFT_CENTER,
                    "▶",
                    FontId::proportional(9.0),
                    pal.text_disabled,
                );
                if resp.double_clicked() {
                    cycle = Some(t.id.clone());
                }
                resp.context_menu(|ui| {
                    if ui.button("编辑标题").clicked() {
                        app.new_name = t.title.clone();
                        ui.close();
                    }
                    if ui.button(RichText::new("删除任务").color(pal.danger)).clicked() {
                        del_task = Some(t.id.clone());
                        ui.close();
                    }
                });
                y += 66.0;
            }
            ui.add_space(14.0);
        }
    });

    if let Some((id, status)) = move_task {
        if let Some(t) = app.novel.as_mut().and_then(|n| n.tasks.iter_mut().find(|t| t.id == id)) {
            t.status = status;
            app.meta_dirty = true;
        }
    }
    if let Some(id) = cycle {
        if let Some(t) = app.novel.as_mut().and_then(|n| n.tasks.iter_mut().find(|t| t.id == id)) {
            t.status = (t.status + 1) % 3;
            app.meta_dirty = true;
        }
    }
    if let Some(id) = del_task {
        if let Some(n) = app.novel.as_mut() {
            n.tasks.retain(|t| t.id != id);
            app.meta_dirty = true;
        }
    }
}
