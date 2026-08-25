//! 模态对话框：新建小说/章节/卷、重命名、删除确认、导出、导入、关于

use eframe::egui::{self, RichText};

use crate::app::{AppState, DialogKind};
use crate::widgets::{self, Palette};

pub fn show_dialog(app: &mut AppState, ctx: &egui::Context) {
    let kind = app.dialog.clone();
    match kind {
        Some(DialogKind::NewNovel) => new_novel_dialog(app, ctx),
        Some(DialogKind::NewChapter) => new_chapter_dialog(app, ctx),
        Some(DialogKind::NewVolume) => new_volume_dialog(app, ctx),
        Some(DialogKind::RenameChapter(cid)) => rename_dialog(app, ctx, "重命名章节", &cid, None),
        Some(DialogKind::RenameVolume(vid)) => rename_dialog(app, ctx, "重命名卷", &vid, Some(true)),
        Some(DialogKind::DeleteChapter(cid, title)) => delete_chapter_dialog(app, ctx, &cid, &title),
        Some(DialogKind::DeleteVolume(vid, title)) => delete_volume_dialog(app, ctx, &vid, &title),
        Some(DialogKind::DeleteNovel(id, title)) => delete_novel_dialog(app, ctx, &id, &title),
        Some(DialogKind::Export) => export_dialog(app, ctx),
        Some(DialogKind::Import) => import_dialog(app, ctx),
        Some(DialogKind::About) => about_dialog(app, ctx),
        None => {}
    }
}

// ---------- 新建小说 ----------
fn new_novel_dialog(app: &mut AppState, ctx: &egui::Context) {
    let pal = app.pal;
    if let Some(_) = widgets::modal(ctx, "dlg_new_novel", "📝 新建小说", egui::Vec2::new(460.0, 420.0), &pal, |ui| {
        ui.label(RichText::new("书名").size(12.5).color(pal.text_secondary));
        widgets::text_input(ui, &mut app.form_title, "例如：剑出昆仑", &pal);
        ui.add_space(6.0);
        ui.label(RichText::new("作者").size(12.5).color(pal.text_secondary));
        widgets::text_input(ui, &mut app.form_author, "你的笔名", &pal);
        ui.add_space(6.0);
        ui.label(RichText::new("题材").size(12.5).color(pal.text_secondary));
        widgets::text_input(ui, &mut app.form_genre, "例如：仙侠 / 都市 / 玄幻 / 悬疑", &pal);
        ui.add_space(6.0);
        ui.label(RichText::new("简介 / 核心设定").size(12.5).color(pal.text_secondary));
        widgets::text_area(ui, &mut app.form_desc, "一句话讲清这本书要写什么……", 4, &pal);
        ui.add_space(14.0);
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let can = !app.form_title.trim().is_empty();
                if widgets::primary_btn(ui, "创建", &pal).clicked() && can {
                    let mut novel = crate::model::Novel::new(
                        &app.form_title,
                        &app.form_author,
                        &app.form_desc,
                        &app.form_genre,
                    );
                    novel.add_volume("正文");
                    let id = novel.meta.id.clone();
                    app.save_novel_meta_silent(&mut novel);
                    app.form_title.clear();
                    app.form_author.clear();
                    app.form_genre.clear();
                    app.form_desc.clear();
                    app.dialog = None;
                    app.open_novel(&id);
                    app.show_toast("小说已创建（加密存储）", true);
                }
                ui.add_space(8.0);
                if widgets::secondary_btn(ui, "取消", &pal).clicked() {
                    app.dialog = None;
                }
            });
        });
    }) {
    } else {
        app.dialog = None;
    }
}

// ---------- 新建章节 ----------
fn new_chapter_dialog(app: &mut AppState, ctx: &egui::Context) {
    let pal = app.pal;
    if let Some(_) = widgets::modal(ctx, "dlg_new_chapter", "📄 新建章节", egui::Vec2::new(420.0, 240.0), &pal, |ui| {
        let vol_target = app.sel_chain.clone();
        let volumes: Vec<(String, String)> = app
            .novel
            .as_ref()
            .map(|n| n.volumes.iter().map(|v| (v.id.clone(), v.title.clone())).collect())
            .unwrap_or_default();
        let mut vol_id = vol_target.clone().unwrap_or_default();
        if vol_id.is_empty() {
            vol_id = volumes.first().map(|(id, _)| id.clone()).unwrap_or_default();
        }
        ui.label(RichText::new("章节标题").size(12.5).color(pal.text_secondary));
        widgets::text_input(ui, &mut app.form_title, "例如：第一章 少年出山", &pal);
        ui.add_space(6.0);
        ui.label(RichText::new("所属卷").size(12.5).color(pal.text_secondary));
        egui::ComboBox::from_id_salt("new_ch_vol")
            .selected_text(
                volumes
                    .iter()
                    .find(|(id, _)| *id == vol_id)
                    .map(|(_, t)| t.clone())
                    .unwrap_or_else(|| "无卷（自动创建）".into()),
            )
            .width(200.0)
            .show_ui(ui, |ui| {
                for (vid, vtitle) in &volumes {
                    if ui.selectable_label(vol_id == *vid, vtitle).clicked() {
                        vol_id = vid.clone();
                    }
                }
            });
        ui.add_space(14.0);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let can = !app.form_title.trim().is_empty();
            if widgets::primary_btn(ui, "创建并打开", &pal).clicked() && can {
                let title = app.form_title.trim().to_string();
                let vid = if volumes.is_empty() { None } else { Some(vol_id.clone()) };
                let new_cid = if let Some(n) = app.novel.as_mut() {
                    if volumes.is_empty() && n.volumes.is_empty() {
                        n.add_volume("正文");
                    }
                    Some(n.add_chapter(vid.as_deref(), &title))
                } else {
                    None
                };
                if let Some(cid) = new_cid {
                    if let Some(novel) = app.novel.as_ref() {
                        app.save_novel_meta_silent(novel);
                    }
                    app.chapters.insert(cid.clone(), String::new());
                    app.dirty.insert(cid.clone());
                    app.open_tab(&cid);
                    app.activity = crate::app::Activity::Chapters;
                    app.show_toast("章节已创建", true);
                }
                app.form_title.clear();
                app.sel_chain = None;
                app.dialog = None;
            }
            ui.add_space(8.0);
            if widgets::secondary_btn(ui, "取消", &pal).clicked() {
                app.dialog = None;
                app.sel_chain = None;
            }
        });
    }) {
    } else {
        app.dialog = None;
    }
}

// ---------- 新建卷 ----------
fn new_volume_dialog(app: &mut AppState, ctx: &egui::Context) {
    let pal = app.pal;
    if let Some(_) = widgets::modal(ctx, "dlg_new_vol", "📁 新建分卷", egui::Vec2::new(380.0, 180.0), &pal, |ui| {
        ui.label(RichText::new("卷名").size(12.5).color(pal.text_secondary));
        widgets::text_input(ui, &mut app.form_title, "例如：第一卷 风起青萍", &pal);
        ui.add_space(14.0);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let can = !app.form_title.trim().is_empty();
            if widgets::primary_btn(ui, "创建", &pal).clicked() && can {
                if let Some(n) = app.novel.as_mut() {
                    n.add_volume(&app.form_title);
                    app.meta_dirty = true;
                }
                app.form_title.clear();
                app.dialog = None;
            }
            ui.add_space(8.0);
            if widgets::secondary_btn(ui, "取消", &pal).clicked() {
                app.dialog = None;
            }
        });
    }) {
    } else {
        app.dialog = None;
    }
}

// ---------- 重命名 ----------
fn rename_dialog(app: &mut AppState, ctx: &egui::Context, title: &str, id: &str, _is_vol: Option<bool>) {
    let pal = app.pal;
    let current = if _is_vol == Some(true) {
        app.novel
            .as_ref()
            .and_then(|n| n.volumes.iter().find(|v| v.id == id))
            .map(|v| v.title.clone())
            .unwrap_or_default()
    } else {
        app.novel
            .as_ref()
            .and_then(|n| n.find_chapter(id))
            .map(|c| c.title.clone())
            .unwrap_or_default()
    };
    if app.form_title.is_empty() {
        app.form_title = current.clone();
    }
    if let Some(_) = widgets::modal(ctx, &format!("dlg_rename_{}", id), title, egui::Vec2::new(380.0, 180.0), &pal, |ui| {
        ui.label(RichText::new("新名称").size(12.5).color(pal.text_secondary));
        widgets::text_input(ui, &mut app.form_title, "", &pal);
        ui.add_space(14.0);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let can = !app.form_title.trim().is_empty();
            if widgets::primary_btn(ui, "确定", &pal).clicked() && can {
                let new_name = app.form_title.trim().to_string();
                if _is_vol == Some(true) {
                    if let Some(v) = app.novel.as_mut().and_then(|n| n.volumes.iter_mut().find(|v| v.id == id)) {
                        v.title = new_name;
                        app.meta_dirty = true;
                    }
                } else if let Some(c) = app.novel.as_mut().and_then(|n| n.find_chapter_mut(id)) {
                    c.title = new_name;
                    app.meta_dirty = true;
                }
                app.form_title.clear();
                app.dialog = None;
            }
            ui.add_space(8.0);
            if widgets::secondary_btn(ui, "取消", &pal).clicked() {
                app.form_title.clear();
                app.dialog = None;
            }
        });
    }) {
    } else {
        app.form_title.clear();
        app.dialog = None;
    }
}

// ---------- 删除确认 ----------
fn delete_chapter_dialog(app: &mut AppState, ctx: &egui::Context, cid: &str, title: &str) {
    let pal = app.pal;
    if let Some(_) = widgets::modal(ctx, "dlg_del_ch", "🗑 删除章节", egui::Vec2::new(380.0, 170.0), &pal, |ui| {
        ui.label(
            RichText::new(format!("确定删除章节《{}》吗？此操作不可恢复。", title))
                .size(13.0)
                .color(pal.text),
        );
        ui.add_space(16.0);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if widgets::primary_btn(ui, "删除", &pal).clicked() {
                let nid = app.novel.as_ref().map(|n| n.meta.id.clone()).unwrap_or_default();
                if let Some(n) = app.novel.as_mut() {
                    n.delete_chapter(cid);
                }
                if let Some(novel) = app.novel.as_ref() {
                    app.save_novel_meta_silent(novel);
                }
                app.chapters.remove(cid);
                app.dirty.remove(cid);
                app.open_tabs.retain(|c| c != cid);
                if app.active_tab.as_deref() == Some(cid) {
                    app.active_tab = app.open_tabs.last().cloned();
                }
                app.store().delete_chapter_file(&nid, cid);
                app.dialog = None;
                app.show_toast("章节已删除", true);
            }
            ui.add_space(8.0);
            if widgets::secondary_btn(ui, "取消", &pal).clicked() {
                app.dialog = None;
            }
        });
    }) {
    } else {
        app.dialog = None;
    }
}

fn delete_volume_dialog(app: &mut AppState, ctx: &egui::Context, vid: &str, title: &str) {
    let pal = app.pal;
    if let Some(_) = widgets::modal(ctx, "dlg_del_vol", "🗑 删除卷", egui::Vec2::new(400.0, 170.0), &pal, |ui| {
        ui.label(
            RichText::new(format!("确定删除卷《{}》及其全部章节吗？此操作不可恢复。", title))
                .size(13.0)
                .color(pal.text),
        );
        ui.add_space(16.0);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if widgets::primary_btn(ui, "删除", &pal).clicked() {
                let nid = app.novel.as_ref().map(|n| n.meta.id.clone()).unwrap_or_default();
                let mut removed: Vec<String> = Vec::new();
                if let Some(n) = app.novel.as_mut() {
                    if let Some(pos) = n.volumes.iter().position(|v| v.id == vid) {
                        removed = n.volumes[pos].chapters.iter().map(|c| c.id.clone()).collect();
                        n.volumes.remove(pos);
                        n.meta.chapter_count = n.chapters_all().len() as u32;
                    }
                }
                if let Some(novel) = app.novel.as_ref() {
                    app.save_novel_meta_silent(novel);
                }
                for cid in &removed {
                    app.chapters.remove(cid);
                    app.dirty.remove(cid);
                    app.open_tabs.retain(|c| c != cid);
                    if app.active_tab.as_deref() == Some(cid.as_str()) {
                        app.active_tab = app.open_tabs.last().cloned();
                    }
                    app.store().delete_chapter_file(&nid, cid);
                }
                app.dialog = None;
                app.show_toast("卷已删除", true);
            }
            ui.add_space(8.0);
            if widgets::secondary_btn(ui, "取消", &pal).clicked() {
                app.dialog = None;
            }
        });
    }) {
    } else {
        app.dialog = None;
    }
}

fn delete_novel_dialog(app: &mut AppState, ctx: &egui::Context, id: &str, title: &str) {
    let pal = app.pal;
    if let Some(_) = widgets::modal(ctx, "dlg_del_novel", "🗑 删除小说", egui::Vec2::new(400.0, 170.0), &pal, |ui| {
        ui.label(
            RichText::new(format!("确定删除《{}》吗？全部章节数据将被清除，且不可恢复。", title))
                .size(13.0)
                .color(pal.text),
        );
        ui.add_space(16.0);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if widgets::primary_btn(ui, "删除", &pal).clicked() {
                match app.store().delete_novel(id) {
                    Ok(()) => {
                        if app.novel.as_ref().map(|n| n.meta.id.as_str()) == Some(id) {
                            app.close_novel();
                        }
                        app.library = app.store().list_novels();
                        app.show_toast("已删除", true);
                    }
                    Err(e) => app.show_toast(&e, false),
                }
                app.dialog = None;
            }
            ui.add_space(8.0);
            if widgets::secondary_btn(ui, "取消", &pal).clicked() {
                app.dialog = None;
            }
        });
    }) {
    } else {
        app.dialog = None;
    }
}

// ---------- 导出 ----------
fn export_dialog(app: &mut AppState, ctx: &egui::Context) {
    let pal = app.pal;
    let Some(novel) = app.novel.as_ref().cloned() else {
        app.dialog = None;
        return;
    };
    if let Some(_) = widgets::modal(ctx, "dlg_export", "📤 导出作品", egui::Vec2::new(440.0, if app.export_fmt == "jsb" { 340.0 } else { 250.0 }), &pal, |ui| {
        ui.label(RichText::new("格式").size(12.5).color(pal.text_secondary));
        ui.horizontal(|ui| {
            for (f, label, hint) in [
                ("txt", "纯文本 (.txt)", "通用，可导入任何写作平台"),
                ("md", "Markdown (.md)", "保留卷/章标题结构"),
                ("jsb", "加密备份 (.jsb)", "密码保护，可跨设备恢复"),
            ] {
                if widgets::pill(ui, label, app.export_fmt == f, &pal).clicked() {
                    app.export_fmt = f.into();
                }
                ui.label(RichText::new(hint).size(10.0).color(pal.text_disabled));
                ui.add_space(4.0);
            }
        });
        if app.export_fmt == "jsb" {
            ui.add_space(8.0);
            ui.label(RichText::new("设置密码（Scrypt 密钥派生，密码不落盘）").size(12.0).color(pal.text_secondary));
            widgets::text_input(ui, &mut app.jsb_pwd, "密码", &pal);
            ui.add_space(4.0);
            widgets::text_input(ui, &mut app.jsb_pwd2, "再次输入密码", &pal);
            if !app.jsb_pwd.is_empty() && app.jsb_pwd != app.jsb_pwd2 {
                ui.label(RichText::new("两次密码不一致").size(11.5).color(pal.danger));
            }
        }
        ui.add_space(14.0);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let can = app.export_fmt != "jsb" || (!app.jsb_pwd.is_empty() && app.jsb_pwd == app.jsb_pwd2);
            if widgets::primary_btn(ui, "导出", &pal).clicked() && can {
                do_export(app, &novel);
                app.dialog = None;
            }
            ui.add_space(8.0);
            if widgets::secondary_btn(ui, "取消", &pal).clicked() {
                app.dialog = None;
            }
        });
    }) {
    } else {
        app.dialog = None;
    }
}

fn do_export(app: &mut AppState, novel: &crate::model::Novel) {
    let chapters: Vec<(String, String, String)> = novel
        .chapters_all()
        .iter()
        .map(|c| {
            (
                c.id.clone(),
                c.title.clone(),
                app.chapters.get(&c.id).cloned().unwrap_or_default(),
            )
        })
        .collect();
    let ext = app.export_fmt.clone();
    let default_path = crate::export::default_export_dir(app.store())
        .join(format!("{}.{}", safe_name(&novel.meta.title), ext));
    let picked = rfd::FileDialog::new()
        .set_title("导出作品")
        .set_file_name(format!("{}.{}", safe_name(&novel.meta.title), ext))
        .set_directory(app.store().data_dir.clone())
        .add_filter(ext.to_uppercase(), &[&ext])
        .save_file();
    let path = picked.unwrap_or(default_path);

    let result = match ext.as_str() {
        "txt" => crate::export::export_txt(novel, &chapters, &path),
        "md" => crate::export::export_md(novel, &chapters, &path),
        _ => {
            let pwd = std::mem::take(&mut app.jsb_pwd);
            app.jsb_pwd2.clear();
            crate::export::export_jsb(&pwd, novel, &chapters, &path)
        }
    };
    match result {
        Ok(()) => app.show_toast(&format!("已导出：{}", path.display()), true),
        Err(e) => app.show_toast(&format!("导出失败：{}", e), false),
    }
}

// ---------- 导入 ----------
fn import_dialog(app: &mut AppState, ctx: &egui::Context) {
    let pal = app.pal;
    if let Some(_) = widgets::modal(ctx, "dlg_import", "📥 导入 .jsb 备份", egui::Vec2::new(420.0, 240.0), &pal, |ui| {
        ui.label(RichText::new("密码").size(12.5).color(pal.text_secondary));
        widgets::text_input(ui, &mut app.jsb_pwd, "备份文件的密码", &pal);
        ui.add_space(12.0);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if widgets::primary_btn(ui, "选择文件并导入", &pal).clicked() && !app.jsb_pwd.is_empty() {
                let picked = rfd::FileDialog::new()
                    .set_title("选择 .jsb 备份")
                    .add_filter("JSB 备份", &["jsb"])
                    .pick_file();
                if let Some(path) = picked {
                    let pwd = std::mem::take(&mut app.jsb_pwd);
                    match std::fs::read(&path) {
                        Ok(bytes) => match crate::export::import_jsb(&pwd, &bytes) {
                            Ok((mut novel, chapters)) => {
                                match crate::export::import_to_store(app.store(), &mut novel, &chapters) {
                                    Ok(id) => {
                                        app.library = app.store().list_novels();
                                        app.dialog = None;
                                        app.open_novel(&id);
                                        app.show_toast("备份已导入", true);
                                    }
                                    Err(e) => app.show_toast(&format!("导入失败：{}", e), false),
                                }
                            }
                            Err(e) => app.show_toast(&format!("解密失败：{}", e), false),
                        },
                        Err(e) => app.show_toast(&format!("读取文件失败：{}", e), false),
                    }
                }
            }
            ui.add_space(8.0);
            if widgets::secondary_btn(ui, "取消", &pal).clicked() {
                app.dialog = None;
            }
        });
    }) {
    } else {
        app.dialog = None;
    }
}

// ---------- 关于 ----------
fn about_dialog(app: &mut AppState, ctx: &egui::Context) {
    let pal = app.pal;
    if let Some(_) = widgets::modal(ctx, "dlg_about", "关于锦书", egui::Vec2::new(420.0, 300.0), &pal, |ui| {
        ui.label(RichText::new("📖 锦书 · 小说编辑器 v0.1.0").size(16.0).strong().color(pal.text));
        ui.add_space(8.0);
        ui.label(RichText::new("为中文小说创作而生的本地编辑器。").size(13.0).color(pal.text));
        ui.add_space(4.0);
        ui.label(
            RichText::new("• 数据 AES-256-GCM 加密存储于安装目录，不写入系统目录\n• 支持 txt / md 导出与 .jsb 密码加密备份\n• 接入任意 OpenAI 兼容 / Anthropic API，大纲、续写、润色、评审一站式\n• 章节树 / 大纲 / 人物关系网 / 世界观 / 时间线 / 任务看板 / 写作统计\n• 跨平台：Windows 11 与 Arch Linux")
                .size(12.0)
                .color(pal.text_secondary),
        );
        ui.add_space(10.0);
        ui.label(
            RichText::new("技术栈：Rust + egui 0.36 · 开源协议 MIT")
                .size(11.0)
                .color(pal.text_disabled),
        );
        ui.add_space(12.0);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if widgets::primary_btn(ui, "好的", &pal).clicked() {
                app.dialog = None;
            }
        });
    }) {
    } else {
        app.dialog = None;
    }
}

fn safe_name(s: &str) -> String {
    let cleaned: String = s
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c.is_whitespace() || "_-（）()·.".contains(c) {
                c
            } else {
                '_'
            }
        })
        .collect();
    if cleaned.trim().is_empty() {
        "未命名".into()
    } else {
        cleaned.trim().to_string()
    }
}

pub fn _unused(_: &Palette) {}
