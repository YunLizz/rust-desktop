//! 设置视图（中央区）：外观 / 编辑 / AI 服务 / 存储

use eframe::egui::{self, CornerRadius, Frame, Margin, RichText, Stroke};

use crate::app::AppState;
use crate::theme;
use crate::widgets::{self, Palette};

pub fn show(ui: &mut egui::Ui, app: &mut AppState) {
    let pal = app.pal;
    ui.add_space(14.0);
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui.label(RichText::new("⚙️ 设置").size(20.0).strong().color(pal.text));
    });
    ui.add_space(10.0);

    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        ui.add_space(8.0);
        section(ui, "外观", &pal, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("主题").size(12.5).color(pal.text_secondary));
                let dark = app.settings.theme == "dark";
                if widgets::pill(ui, "深色", dark, &pal).clicked() {
                    app.settings.theme = "dark".into();
                    app.refresh_palette();
                    let _ = app.settings_save();
                }
                if widgets::pill(ui, "浅色", !dark, &pal).clicked() {
                    app.settings.theme = "light".into();
                    app.refresh_palette();
                    let _ = app.settings_save();
                }
            });
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("强调色").size(12.5).color(pal.text_secondary));
                for (name, rgb) in theme::ACCENT_PRESETS {
                    let selected = app.settings.accent == rgb;
                    let (rect, resp) = ui.allocate_exact_size(
                        eframe::egui::Vec2::new(26.0, 26.0),
                        eframe::egui::Sense::click(),
                    );
                    ui.painter().rect_filled(
                        rect,
                        CornerRadius::same(8),
                        eframe::egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]),
                    );
                    if selected {
                        ui.painter().rect_stroke(
                            rect,
                            CornerRadius::same(8),
                            Stroke::new(2.0, pal.text),
 egui::StrokeKind::Middle);
                    }
                    if resp.clicked() {
                        app.settings.accent = rgb;
                        app.refresh_palette();
                        let _ = app.settings_save();
                    }
                    resp.on_hover_text(name);
                }
            });
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("界面缩放").size(12.5).color(pal.text_secondary));
                ui.add(eframe::egui::Slider::new(&mut app.settings.ui_scale, 0.8..=1.6).text("倍"));
            });
        });
        ui.add_space(10.0);

        section(ui, "编辑", &pal, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("正文字体").size(12.5).color(pal.text_secondary));
                let serif = app.settings.editor.font == "serif";
                if widgets::pill(ui, "宋体类（衬线）", serif, &pal).clicked() {
                    app.settings.editor.font = "serif".into();
                    app.need_font_reload = true;
                    let _ = app.settings_save();
                }
                if widgets::pill(ui, "黑体类（无衬线）", !serif, &pal).clicked() {
                    app.settings.editor.font = "sans".into();
                    app.need_font_reload = true;
                    let _ = app.settings_save();
                }
            });
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("字号").size(12.5).color(pal.text_secondary));
                ui.add(eframe::egui::Slider::new(&mut app.settings.editor.font_size, 12.0..=32.0).text("px"));
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new("行距").size(12.5).color(pal.text_secondary));
                ui.add(eframe::egui::Slider::new(&mut app.settings.editor.line_spacing, 1.0..=2.8).text("倍"));
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new("自动换行").size(12.5).color(pal.text_secondary));
                widgets::toggle(ui, &mut app.settings.editor.wrap, &pal);
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new("行号").size(12.5).color(pal.text_secondary));
                widgets::toggle(ui, &mut app.settings.editor.show_line_numbers, &pal);
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new("Markdown 高亮").size(12.5).color(pal.text_secondary));
                widgets::toggle(ui, &mut app.settings.editor.markdown_highlight, &pal);
                ui.label(RichText::new("（中文写作建议关闭，# * 符号会原样显示）").size(10.5).color(pal.text_disabled));
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new("自动保存间隔").size(12.5).color(pal.text_secondary));
                ui.add(eframe::egui::Slider::new(&mut app.settings.autosave_secs, 1..=120).text("秒"));
            });
        });
        ui.add_space(10.0);

        section(ui, "AI 服务", &pal, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("协议").size(12.5).color(pal.text_secondary));
                if widgets::pill(ui, "OpenAI 兼容", app.settings.ai.protocol != "anthropic", &pal).clicked() {
                    app.settings.ai.protocol = "openai".into();
                    if app.settings.ai.base_url.is_empty() || app.settings.ai.base_url.contains("openai") {
                        app.settings.ai.base_url = "https://api.openapp.settings.ai.com/v1".into();
                        app.settings.ai.model = "gpt-4o-mini".into();
                    }
                    let _ = app.settings_save();
                }
                if widgets::pill(ui, "Anthropic", app.settings.ai.protocol == "anthropic", &pal).clicked() {
                    app.settings.ai.protocol = "anthropic".into();
                    if app.settings.ai.base_url.is_empty() || !app.settings.ai.base_url.contains("anthropic") {
                        app.settings.ai.base_url = "https://api.anthropic.com".into();
                        app.settings.ai.model = "claude-sonnet-4-5".into();
                    }
                    let _ = app.settings_save();
                }
            });
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("服务商预设").size(12.5).color(pal.text_secondary));
                for (name, proto, url, model) in [
                    ("DeepSeek", "openai", "https://api.deepseek.com/v1", "deepseek-chat"),
                    ("Moonshot", "openai", "https://api.moonshot.cn/v1", "moonshot-v1-8k"),
                    ("通义千问", "openai", "https://dashscope.aliyuncs.com/compatible-mode/v1", "qwen-plus"),
                    ("智谱 GLM", "openai", "https://open.bigmodel.cn/api/paas/v4", "glm-4-flash"),
                    ("Ollama 本地", "openai", "http://localhost:11434/v1", "qwen2.5"),
                    ("自定义", "", "", ""),
                ] {
                    if widgets::pill(ui, name, false, &pal).clicked() && !proto.is_empty() {
                        app.settings.ai.protocol = proto.into();
                        app.settings.ai.base_url = url.into();
                        app.settings.ai.model = model.into();
                        let _ = app.settings_save();
                    }
                }
            });
            ui.add_space(6.0);
            row(ui, "Base URL", &pal, |ui| {
                let resp = widgets::text_input(ui, &mut app.settings.ai.base_url, "https://…/v1", &pal);
                if resp.changed() {
                    let _ = app.settings_save();
                }
            });
            row(ui, "模型", &pal, |ui| {
                let resp = widgets::text_input(ui, &mut app.settings.ai.model, "模型名称", &pal);
                if resp.changed() {
                    let _ = app.settings_save();
                }
            });
            row(ui, "API Key", &pal, |ui| {
                ui.horizontal(|ui| {
                    let showing = app.new_name == "__show_key__";
                    if showing {
                        if widgets::text_input(ui, &mut app.settings.ai.api_key, "sk-…", &pal).changed() {
                            let _ = app.settings_save();
                        }
                    } else if app.settings.ai.api_key.is_empty() {
                        ui.label(RichText::new("（未填写）").size(12.5).color(pal.text_disabled));
                    } else {
                        let tail: String = app.settings.ai.api_key.chars().rev().take(4).collect::<String>().chars().rev().collect();
                        ui.label(
                            RichText::new(format!("••••••••••••{}", tail))
                                .size(12.5)
                                .color(pal.text_secondary),
                        );
                    }
                    if widgets::icon_btn(ui, if showing { "🙈" } else { "👁" }, "显示/隐藏", &pal).clicked() {
                        app.new_name = if showing { String::new() } else { "__show_key__".into() };
                    }
                    if widgets::secondary_btn(ui, "测试连接", &pal).clicked() {
                        app.ai_test_rx = Some(ai_test(app.settings.ai.clone()));
                    }
                });
            });
            ui.add_space(4.0);
            if let Some(rx) = &app.ai_test_rx {
                if let Ok(result) = rx.try_recv() {
                    match result {
                        Ok(reply) => {
                            app.show_toast(&format!("连接成功：{}", reply), true);
                            app.ai_test_rx = None;
                        }
                        Err(e) => {
                            app.show_toast(&format!("连接失败：{}", e), false);
                            app.ai_test_rx = None;
                        }
                    }
                }
            }
            ui.horizontal(|ui| {
                ui.label(RichText::new("温度").size(12.5).color(pal.text_secondary));
                ui.add(eframe::egui::Slider::new(&mut app.settings.ai.temperature, 0.0..=1.5).text(""));
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new("最大 Token").size(12.5).color(pal.text_secondary));
                ui.add(eframe::egui::DragValue::new(&mut app.settings.ai.max_tokens).range(256..=32000));
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new("超时（秒）").size(12.5).color(pal.text_secondary));
                ui.add(eframe::egui::DragValue::new(&mut app.settings.ai.timeout_secs).range(30..=900));
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new("自动注入设定").size(12.5).color(pal.text_secondary));
                widgets::toggle(ui, &mut app.settings.ai.inject_lore, &pal);
                ui.label(RichText::new("（正文出现人物/地名时自动附带对应卡片）").size(10.5).color(pal.text_disabled));
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("系统提示词").size(12.5).color(pal.text_secondary));
            });
            ui.horizontal(|ui| {
                ui.add_space(12.0);
                let resp = widgets::text_area(ui, &mut app.settings.ai.system_prompt, "", 3, &pal);
                if resp.changed() {
                    let _ = app.settings_save();
                }
            });
        });
        ui.add_space(10.0);

        section(ui, "存储与安全", &pal, |ui| {
            ui.label(RichText::new(format!("数据目录：{}", app.data_dir)).size(12.0).color(pal.text_secondary));
            ui.label(
                RichText::new("所有作品、章节与设置均以 AES-256-GCM 加密文件存储于此目录，密钥为本机随机生成的 32 字节文件，不写入任何系统目录。")
                    .size(11.0)
                    .color(pal.text_disabled),
            );
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("密钥指纹：{}", app.key_fp)).size(12.0).color(pal.ok));
            });
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if widgets::secondary_btn(ui, "📂 打开数据目录", &pal).clicked() {
                    open_dir(&app.data_dir);
                }
                if widgets::secondary_btn(ui, "📤 导出 .jsb 加密备份", &pal).clicked() {
                    app.dialog = Some(crate::app::DialogKind::Export);
                }
                if widgets::secondary_btn(ui, "📥 导入 .jsb 备份", &pal).clicked() {
                    app.dialog = Some(crate::app::DialogKind::Import);
                }
            });
            ui.add_space(6.0);
            ui.label(
                RichText::new("提示：.jsb 备份使用你设定的密码（Scrypt 派生密钥），密码不落盘，可跨设备恢复；请妥善保管密码。")
                    .size(11.0)
                    .color(pal.text_disabled),
            );
        });
        ui.add_space(16.0);
    });
}

fn row(ui: &mut egui::Ui, label: &str, pal: &Palette, content: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(12.5).color(pal.text_secondary));
        content(ui);
    });
}

fn section(ui: &mut egui::Ui, title: &str, pal: &Palette, content: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui.label(RichText::new(title).size(14.0).strong().color(pal.accent));
    });
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        let inner = Frame::new()
            .fill(pal.bg_panel)
            .stroke(Stroke::new(1.0, pal.border))
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::same(14));
        inner.show(ui, |ui| {
            ui.set_width(ui.available_width() - 8.0);
            content(ui);
        });
    });
    ui.add_space(6.0);
}

fn ai_test(cfg: crate::store::AiSettings) -> std::sync::mpsc::Receiver<Result<String, String>> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(crate::ai::client::test_connection(&cfg));
    });
    rx
}

fn open_dir(path: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer").arg(path).spawn();
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = std::process::Command::new("xdg-open").arg(path).spawn();
    }
}
