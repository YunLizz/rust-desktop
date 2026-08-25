#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ai;
mod ai_panel;
mod app;
mod dialogs;
mod editor;
mod export;
mod model;
mod settings;
mod shell;
mod store;
mod theme;
mod util;
mod views;
mod widgets;

use eframe::egui;

fn main() -> eframe::Result {
    // Windows 使用自定义标题栏（无边框，现代化）；Linux 使用原生装饰（Wayland 兼容）。
    // 设置环境变量 JINSHU_NATIVE_TITLEBAR=1 可强制使用原生标题栏。
    // Windows 使用自定义标题栏（无边框，现代化）；Linux 使用原生装饰（Wayland 兼容）。
    // 设置环境变量 JINSHU_NATIVE_TITLEBAR=1 可强制使用原生标题栏。
    let native_titlebar = std::env::var("JINSHU_NATIVE_TITLEBAR").is_ok() || !cfg!(windows);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1360.0, 860.0])
            .with_min_inner_size([1000.0, 640.0])
            .with_title("锦书 · 小说编辑器")
            .with_decorations(native_titlebar)
            .with_icon(egui::IconData::default()),
        ..Default::default()
    };
    eframe::run_native(
        "jinshu-rust",
        options,
        Box::new(|cc| Ok(Box::new(shell::App::new(cc)))),
    )
}
