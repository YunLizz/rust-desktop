//! 视图：欢迎页 / 书库 / 搜索 / 统计

use eframe::egui::{self, Align, Align2, Color32, CornerRadius, Layout, Rect, RichText, Sense, Stroke, Vec2};

use crate::app::{Activity, AppState, DialogKind};
use crate::util;
use crate::widgets;

pub mod chapters;
pub mod world;

pub use chapters::{chapters_panel, outline_detail, outline_panel};
pub use world::{
    character_detail, characters_panel, location_detail, task_board, tasks_panel,
    timeline_detail, timeline_panel, world_panel,
};

// ---------- 欢迎页 ----------
pub fn welcome_view(ui: &mut egui::Ui, app: &mut AppState) {
    let pal = app.pal;
    let rect = ui.clip_rect();
    let center = rect.center();

    ui.painter().text(
        egui::pos2(center.x, center.y - 130.0),
        Align2::CENTER_CENTER,
        "📖",
        egui::FontId::proportional(52.0),
        pal.accent,
    );
    ui.painter().text(
        egui::pos2(center.x, center.y - 60.0),
        Align2::CENTER_CENTER,
        "锦书",
        egui::FontId::proportional(30.0),
        pal.text,
    );
    ui.painter().text(
        egui::pos2(center.x, center.y - 24.0),
        Align2::CENTER_CENTER,
        "为中文小说创作而生的本地编辑器 · 数据加密存储",
        egui::FontId::proportional(13.5),
        pal.text_secondary,
    );

    // 三个入口卡片
    let card_w = 230.0;
    let card_h = 104.0;
    let gap = 26.0;
    let total = card_w * 3.0 + gap * 2.0;
    let start_x = center.x - total / 2.0;
    let y = center.y + 36.0;

    let actions: [(&str, &str, &str); 3] = [
        ("📝", "新建小说", "创建一部新作品"),
        ("📚", "打开书库", "浏览本地作品库"),
        ("🔑", "导入备份", "从 .jsb 加密备份恢复"),
    ];
    for (i, (icon, title, sub)) in actions.iter().enumerate() {
        let r = Rect::from_min_size(
            egui::pos2(start_x + i as f32 * (card_w + gap), y),
            Vec2::new(card_w, card_h),
        );
        let hover = r.contains(ui.pointer_latest_pos().unwrap_or_default());
        let bg = if hover { pal.bg_hover } else { pal.bg_panel_alt };
        ui.painter().rect_filled(r, CornerRadius::same(12), bg);
        ui.painter().rect_stroke(
            r,
            CornerRadius::same(12),
            Stroke::new(1.0, if hover { pal.accent.gamma_multiply(0.7) } else { pal.border }),
 egui::StrokeKind::Middle);
        ui.painter().text(
            egui::pos2(r.center().x, r.top() + 24.0),
            Align2::CENTER_CENTER,
            *icon,
            egui::FontId::proportional(22.0),
            pal.accent,
        );
        ui.painter().text(
            egui::pos2(r.center().x, r.top() + 56.0),
            Align2::CENTER_CENTER,
            *title,
            egui::FontId::proportional(14.5),
            pal.text,
        );
        ui.painter().text(
            egui::pos2(r.center().x, r.top() + 82.0),
            Align2::CENTER_CENTER,
            *sub,
            egui::FontId::proportional(11.5),
            pal.text_secondary,
        );
        let resp = ui.interact(r, ui.id().with(("welcome", i)), Sense::click());
        if resp.clicked() {
            match i {
                0 => app.dialog = Some(DialogKind::NewNovel),
                1 => {
                    app.activity = Activity::Library;
                    app.library = app.store().list_novels();
                }
                _ => app.dialog = Some(DialogKind::Import),
            }
        }
    }

    // 最近打开
    if !app.settings.recent.is_empty() {
        let y2 = y + card_h + 34.0;
        ui.painter().text(
            egui::pos2(center.x, y2),
            Align2::CENTER_CENTER,
            "最近打开",
            egui::FontId::proportional(12.0),
            pal.text_disabled,
        );
        let mut idx = 0;
        for r in &app.settings.recent.clone() {
            let y3 = y2 + 24.0 + idx as f32 * 22.0;
            let r2 = Rect::from_min_size(
                egui::pos2(center.x - 140.0, y3),
                Vec2::new(280.0, 20.0),
            );
            if ui
                .interact(r2, ui.id().with(("recent", idx)), Sense::click())
                .clicked()
            {
                app.open_novel(&r.id);
            }
            ui.painter().text(
                egui::pos2(r2.center().x, r2.center().y),
                Align2::CENTER_CENTER,
                &r.title,
                egui::FontId::proportional(12.5),
                pal.text_secondary,
            );
            idx += 1;
        }
    }
}

// ---------- 书库 ----------
pub fn library_view(ui: &mut egui::Ui, app: &mut AppState) {
    let pal = app.pal;
    let _ = &pal;
    ui.add_space(14.0);
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui.label(RichText::new("📚 我的书库").size(20.0).strong().color(app.pal.text));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add_space(18.0);
            if widgets::primary_btn(ui, "＋ 新建小说", &app.pal).clicked() {
                app.dialog = Some(DialogKind::NewNovel);
            }
            if widgets::secondary_btn(ui, "导入 .jsb", &app.pal).clicked() {
                app.dialog = Some(DialogKind::Import);
            }
            if widgets::secondary_btn(ui, "刷新", &app.pal).clicked() {
                app.library = app.store().list_novels();
            }
        });
    });
    ui.add_space(10.0);

    if app.library.is_empty() {
        welcome_view(ui, app);
        return;
    }
    let mut delete_id: Option<String> = None;
    let mut open_id: Option<String> = None;
    let mut export_id: Option<String> = None;

    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(14.0, 14.0);
            let lib = app.library.clone();
            for meta in &lib {
                let w = 240.0;
                let h = 130.0;
                let (rect, _) = ui.allocate_exact_size(Vec2::new(w, h), Sense::hover());
                let hover = rect.contains(ui.pointer_latest_pos().unwrap_or_default());
                let bg = if hover { app.pal.bg_panel_alt } else { app.pal.bg_panel };
                ui.painter().rect_filled(rect, CornerRadius::same(12), bg);
                ui.painter().rect_stroke(
                    rect,
                    CornerRadius::same(12),
                    Stroke::new(1.0, if hover { app.pal.accent.gamma_multiply(0.6) } else { app.pal.border }),
 egui::StrokeKind::Middle);
                // 封面区
                let cover = Rect::from_min_size(rect.min, Vec2::new(52.0, h));
                ui.painter().rect_filled(cover, CornerRadius::same(12), app.pal.accent.gamma_multiply(0.25));
                ui.painter().text(
                    cover.center(),
                    Align2::CENTER_CENTER,
                    "📖",
                    egui::FontId::proportional(22.0),
                    app.pal.accent,
                );
                ui.painter().text(
                    egui::pos2(rect.left() + 64.0, rect.top() + 18.0),
                    Align2::LEFT_CENTER,
                    &meta.title,
                    egui::FontId::proportional(14.5),
                    app.pal.text,
                );
                if !meta.author.is_empty() {
                    ui.painter().text(
                        egui::pos2(rect.left() + 64.0, rect.top() + 40.0),
                        Align2::LEFT_CENTER,
                        &meta.author,
                        egui::FontId::proportional(11.5),
                        app.pal.text_secondary,
                    );
                }
                ui.painter().text(
                    egui::pos2(rect.left() + 64.0, rect.bottom() - 34.0),
                    Align2::LEFT_CENTER,
                    format!("{} 章 · {} 字", meta.chapter_count, meta.total_words),
                    egui::FontId::proportional(11.0),
                    app.pal.text_disabled,
                );
                ui.painter().text(
                    egui::pos2(rect.left() + 64.0, rect.bottom() - 16.0),
                    Align2::LEFT_CENTER,
                    format!("更新于 {}", util::format_ts(meta.updated_at)),
                    egui::FontId::proportional(10.5),
                    app.pal.text_disabled,
                );
                let resp = ui.interact(rect, ui.id().with(("book", meta.id.clone())), Sense::click());
                if resp.double_clicked() {
                    open_id = Some(meta.id.clone());
                }
                resp.context_menu(|ui| {
                    if ui.button("打开").clicked() {
                        open_id = Some(meta.id.clone());
                        ui.close();
                    }
                    if ui.button("导出").clicked() {
                        export_id = Some(meta.id.clone());
                        ui.close();
                    }
                    if ui.button(RichText::new("删除").color(app.pal.danger)).clicked() {
                        delete_id = Some(meta.id.clone());
                        ui.close();
                    }
                });
            }
        });
    });

    if let Some(id) = delete_id {
        let title = app
            .library
            .iter()
            .find(|m| m.id == id)
            .map(|m| m.title.clone())
            .unwrap_or_default();
        app.dialog = Some(DialogKind::DeleteNovel(id, title));
    }
    if let Some(id) = open_id {
        app.open_novel(&id);
    }
    if let Some(id) = export_id {
        if app.open_novel_before_export(&id) {
            app.dialog = Some(DialogKind::Export);
        }
    }
}

// ---------- 搜索面板 ----------
pub fn search_panel(ui: &mut egui::Ui, app: &mut AppState) {
    let pal = app.pal;
    widgets::section_header(ui, "全局搜索", &pal);
    let mut query = String::new();
    let resp = widgets::text_input(ui, &mut query, "输入关键词搜索全部章节…", &pal);
    if resp.changed() {
        app.new_name = query.clone();
    }
    let q = app.new_name.clone();
    ui.add_space(4.0);
    widgets::h_sep(ui, &pal);
    ui.add_space(4.0);

    let Some(novel) = app.novel.as_ref() else { return };
    let mut results: Vec<(String, String, String)> = Vec::new(); // cid, title, 行预览
    if !q.trim().is_empty() {
        for c in novel.chapters_all() {
            if let Some(text) = app.chapters.get(&c.id) {
                if let Some(line) = text
                    .lines()
                    .find(|l| l.contains(q.trim()))
                {
                    results.push((c.id.clone(), c.title.clone(), line.trim().to_string()));
                }
            }
        }
    }
    let mut open: Option<String> = None;
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        for (cid, title, line) in &results {
            let (rect, resp) = ui.allocate_exact_size(
                Vec2::new(ui.available_width(), 42.0),
                Sense::click(),
            );
            if resp.hovered() {
                ui.painter().rect_filled(rect, CornerRadius::same(6), pal.bg_hover);
            }
            ui.painter().text(
                egui::pos2(rect.left() + 8.0, rect.top() + 10.0),
                Align2::LEFT_CENTER,
                title,
                egui::FontId::proportional(12.5),
                pal.text,
            );
            ui.painter().text(
                egui::pos2(rect.left() + 8.0, rect.top() + 27.0),
                Align2::LEFT_CENTER,
                format!("…{}…", crate::util::truncate_chars(&line, 40)),
                egui::FontId::proportional(11.0),
                pal.text_secondary,
            );
            if resp.clicked() {
                open = Some(cid.clone());
            }
        }
        if results.is_empty() && !q.trim().is_empty() {
            ui.label(RichText::new("无匹配结果").color(pal.text_disabled));
        }
    });
    if let Some(cid) = open {
        app.open_tab(&cid);
        app.activity = Activity::Chapters;
        app.find = Some(crate::app::FindState {
            query: q.clone(),
            ..crate::app::FindState::new()
        });
        app.dialog = None;
    }
}

// ---------- 统计 ----------
pub fn stats_view(ui: &mut egui::Ui, app: &mut AppState) {
    let pal = app.pal;
    let Some(novel) = app.novel.as_ref() else {
        widgets::empty_state(ui, "📊", "请先打开一部小说", "", &pal);
        return;
    };
    ui.add_space(14.0);
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui.label(RichText::new("📊 写作统计").size(20.0).strong().color(pal.text));
        ui.label(
            RichText::new(format!("《{}》", novel.meta.title))
                .size(13.0)
                .color(pal.text_secondary),
        );
    });
    ui.add_space(10.0);

    let today_words = novel.stats.get(&util::today()).copied().unwrap_or(0);
    let total: u64 = novel.stats.values().sum();
    let streak = calc_streak(&novel.stats);
    let days = novel.stats.len();
    let chapters = novel.chapters_all().len();

    // 数据卡
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        for (label, value, color) in [
            ("总字数", novel.total_words().to_string(), pal.text),
            ("今日新增", format!("+{}", today_words), pal.warn),
            ("累计写作", total.to_string(), pal.accent),
            ("连续创作(天)", streak.to_string(), pal.ok),
            ("创作天数", days.to_string(), pal.purple),
            ("章节数", chapters.to_string(), pal.cyan),
        ] {
            let (rect, _) = ui.allocate_exact_size(Vec2::new(150.0, 76.0), Sense::hover());
            ui.painter().rect_filled(rect, CornerRadius::same(10), pal.bg_panel);
            ui.painter().rect_stroke(rect, CornerRadius::same(10), Stroke::new(1.0, pal.border), egui::StrokeKind::Middle);
            ui.painter().text(
                egui::pos2(rect.center().x, rect.top() + 24.0),
                Align2::CENTER_CENTER,
                &value,
                egui::FontId::proportional(20.0),
                color,
            );
            ui.painter().text(
                egui::pos2(rect.center().x, rect.bottom() - 18.0),
                Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(11.5),
                pal.text_secondary,
            );
            ui.add_space(12.0);
        }
    });

    ui.add_space(20.0);
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui.label(RichText::new("近 30 天字数").size(14.0).strong().color(pal.text));
    });
    ui.add_space(6.0);
    // 柱状图
    egui::ScrollArea::horizontal().show(ui, |ui| {
        ui.add_space(18.0);
        let bar_w = 22.0;
        let bar_gap = 6.0;
        let chart_h = 160.0;
        let mut entries: Vec<(String, u64)> = Vec::new();
        for i in (0..30).rev() {
            let d = util::days_ago(i);
            entries.push((d.clone(), novel.stats.get(&d).copied().unwrap_or(0)));
        }
        let max = entries.iter().map(|(_, v)| *v).max().unwrap_or(1).max(1) as f32;
        let mut x = 0.0;
        let base_y = 140.0;
        ui.allocate_exact_size(Vec2::new(entries.len() as f32 * (bar_w + bar_gap) + 20.0, 190.0), Sense::hover());
        let origin = ui.cursor().min;
        for (i, (date, v)) in entries.iter().enumerate() {
            let h = (*v as f32 / max) * chart_h;
            let r = Rect::from_min_size(
                egui::pos2(origin.x + x, base_y - h),
                Vec2::new(bar_w, h),
            );
            let color = if *v > 0 { pal.accent } else { pal.bg_panel_alt };
            ui.painter().rect_filled(r, CornerRadius::same(3), color);
            if *v > 0 {
                ui.painter().text(
                    egui::pos2(r.center().x, r.top() - 8.0),
                    Align2::CENTER_CENTER,
                    v.to_string(),
                    egui::FontId::proportional(9.0),
                    pal.text_secondary,
                );
            }
            if i % 5 == 0 {
                ui.painter().text(
                    egui::pos2(r.center().x, base_y + 12.0),
                    Align2::CENTER_CENTER,
                    &date[5..],
                    egui::FontId::proportional(9.5),
                    pal.text_disabled,
                );
            }
            x += bar_w + bar_gap;
        }
    });

    ui.add_space(16.0);
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui.label(RichText::new("各卷字数").size(14.0).strong().color(pal.text));
    });
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        for v in &novel.volumes {
            let words: u64 = v.chapters.iter().map(|c| c.words).sum();
            ui.label(
                RichText::new(format!("{}：{} 字", v.title, words))
                    .size(12.0)
                    .color(pal.text_secondary),
            );
        }
    });
    ui.add_space(10.0);
}

fn calc_streak(stats: &std::collections::BTreeMap<String, u64>) -> u32 {
    let mut streak = 0;
    let mut day = chrono::Local::now().naive_local().date();
    // 今天没写也算连续（从今天或昨天开始数）
    if stats.get(&day.format("%Y-%m-%d").to_string()).is_none() {
        day = day.pred_opt().unwrap_or(day);
    }
    loop {
        let key = day.format("%Y-%m-%d").to_string();
        if stats.get(&key).map(|v| *v > 0).unwrap_or(false) {
            streak += 1;
            day = day.pred_opt().unwrap_or(day);
        } else {
            break;
        }
        if streak > 2000 {
            break;
        }
    }
    streak
}
