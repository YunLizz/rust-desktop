//! 通用 UI 组件库：现代化 IDE 风格的按钮 / 导航 / 卡片 / 开关 / 模态框
//! 设计规范：4px 网格、4/6/8/10 圆角阶梯、低对比度边框、单色强调色

use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, Frame, Id, Layout, Margin, Rect, RichText, Sense,
    Stroke, Vec2,
};

pub use crate::theme::Palette;

// ---------- 图标（emoji 字体，egui 自带子集） ----------
pub const IC_EXPLORER: &str = "📚";
pub const IC_SEARCH: &str = "🔍";
pub const IC_AI: &str = "✨";
pub const IC_STATS: &str = "📊";
pub const IC_SETTINGS: &str = "⚙️";
pub const IC_PLUS: &str = "＋";
pub const IC_CLOSE: &str = "✕";
pub const IC_MIN: &str = "─";
pub const IC_MAX: &str = "□";
pub const IC_CHAT: &str = "💬";
pub const IC_OUTLINE: &str = "🗂";
pub const IC_PEOPLE: &str = "👥";
pub const IC_MAP: &str = "🗺";
pub const IC_TASKS: &str = "🎯";
pub const IC_SAVE: &str = "💾";
pub const IC_EXPORT: &str = "📤";
// 预留图标
#[allow(dead_code)] pub const IC_EDIT: &str = "✏️";
#[allow(dead_code)] pub const IC_TRASH: &str = "🗑";
#[allow(dead_code)] pub const IC_PLAY: &str = "▶";
#[allow(dead_code)] pub const IC_STOP: &str = "⏹";
// 预留图标：当前模块未直接使用，但作为 widgets 对外公开 API，保留给调用方或后续功能使用
#[allow(dead_code)] pub const IC_FOLDER: &str = "📁";
#[allow(dead_code)] pub const IC_FILE: &str = "📄";
#[allow(dead_code)] pub const IC_SEND: &str = "➤";
#[allow(dead_code)] pub const IC_BACK: &str = "◀";
#[allow(dead_code)] pub const IC_CHEVRON_R: &str = "›";
#[allow(dead_code)] pub const IC_CHEVRON_D: &str = "⌄";
#[allow(dead_code)] pub const IC_DOT: &str = "•";
#[allow(dead_code)] pub const IC_LOCK: &str = "🔒";
#[allow(dead_code)] pub const IC_LINK: &str = "🔗";

/// 圆角图标按钮
pub fn icon_btn(ui: &mut egui::Ui, icon: &str, tip: &str, pal: &Palette) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(26.0), Sense::click());
    let hover = resp.hovered() || resp.contains_pointer();
    let bg = if hover {
        ui.ctx().animate_bool_with_time(ui.id().with(icon), true, 0.1);
        pal.bg_hover
    } else {
        pal.bg_panel_alt
    };
    if bg != Color32::TRANSPARENT {
        ui.painter().rect_filled(rect, CornerRadius::same(6), bg);
    }
    let color = if resp.hovered() { pal.text } else { pal.text_secondary };
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        icon,
        egui::FontId::proportional(14.0),
        color,
    );
    if !tip.is_empty() {
        resp.on_hover_text(tip)
    } else {
        resp
    }
}

/// 活动栏大图标按钮（带选中指示条）
pub fn activity_btn(
    ui: &mut egui::Ui,
    icon: &str,
    tip: &str,
    selected: bool,
    pal: &Palette,
) -> egui::Response {
    let size = 44.0;
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(size, size), Sense::click());
    let hover = resp.hovered();
    if hover && !selected {
        ui.painter().rect_filled(rect, CornerRadius::same(10), pal.bg_hover);
    }
    if selected {
        ui.painter().rect_filled(rect, CornerRadius::same(10), pal.accent.gamma_multiply(0.18));
        ui.painter().rect_filled(
            Rect::from_min_size(rect.min, Vec2::new(3.0, rect.height())),
            CornerRadius::same(2),
            pal.accent,
        );
    }
    let color = if selected {
        pal.accent
    } else if hover {
        pal.text
    } else {
        Color32::from_rgb(
            (pal.text_secondary.r() as u16 + 40).min(255) as u8,
            (pal.text_secondary.g() as u16 + 40).min(255) as u8,
            (pal.text_secondary.b() as u16 + 40).min(255) as u8,
        )
    };
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        icon,
        egui::FontId::proportional(18.0),
        color,
    );
    if !tip.is_empty() {
        resp.on_hover_text(tip)
    } else {
        resp
    }
}

/// 侧边栏导航项（图标 + 文字，选中高亮）
#[allow(dead_code)]
pub fn nav_item(
    ui: &mut egui::Ui,
    icon: &str,
    label: &str,
    selected: bool,
    pal: &Palette,
) -> egui::Response {
    let height = 28.0;
    let width = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());
    let hover = resp.hovered();
    if hover && !selected {
        ui.painter().rect_filled(rect, CornerRadius::same(6), pal.bg_hover);
    }
    if selected {
        ui.painter().rect_filled(rect, CornerRadius::same(6), pal.accent.gamma_multiply(0.16));
    }
    let icon_color = if selected { pal.accent } else { pal.text_secondary };
    let text_color = if selected { pal.text } else if hover { pal.text } else { pal.text_secondary };
    let text_rect = rect.shrink2(Vec2::new(8.0, 0.0));
    ui.painter().text(
        egui::pos2(text_rect.left() + 2.0, rect.center().y),
        Align2::LEFT_CENTER,
        icon,
        egui::FontId::proportional(12.5),
        icon_color,
    );
    ui.painter().text(
        egui::pos2(text_rect.left() + 24.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(13.5),
        text_color,
    );
    resp
}

/// 胶囊按钮（分段控件 / 标签）
pub fn pill(ui: &mut egui::Ui, label: &str, selected: bool, pal: &Palette) -> egui::Response {
    let text = RichText::new(label)
        .size(12.5)
        .color(if selected { pal.text } else { pal.text_secondary });
    let btn = egui::Button::new(text)
        .fill(if selected { pal.accent.gamma_multiply(0.22) } else { pal.bg_panel_alt })
        .stroke(if selected { Stroke::new(1.0, pal.accent.gamma_multiply(0.6)) } else { Stroke::new(1.0, pal.border) })
        .corner_radius(CornerRadius::same(14))
        .min_size(Vec2::new(0.0, 24.0));
    ui.add(btn)
}

/// 主按钮（强调色填充）
pub fn primary_btn(ui: &mut egui::Ui, label: &str, pal: &Palette) -> egui::Response {
    let text = RichText::new(label).size(13.5).color(pal.text);
    ui.add(
        egui::Button::new(text)
            .fill(pal.accent.gamma_multiply(0.85))
            .stroke(Stroke::NONE)
            
            .min_size(Vec2::new(0.0, 30.0)),
    )
}

/// 次级按钮
pub fn secondary_btn(ui: &mut egui::Ui, label: &str, pal: &Palette) -> egui::Response {
    let text = RichText::new(label).size(13.0).color(pal.text);
    ui.add(
        egui::Button::new(text)
            .fill(pal.bg_panel_alt)
            .stroke(Stroke::new(1.0, pal.border))
            
            .min_size(Vec2::new(0.0, 28.0)),
    )
}

/// 危险按钮（红色描边）
#[allow(dead_code)]
pub fn danger_btn(ui: &mut egui::Ui, label: &str, pal: &Palette) -> egui::Response {
    let text = RichText::new(label).size(13.0).color(pal.danger);
    ui.add(
        egui::Button::new(text)
            .fill(Color32::TRANSPARENT)
            .stroke(Stroke::new(1.0, pal.danger.gamma_multiply(0.5)))
            
            .min_size(Vec2::new(0.0, 28.0)),
    )
}

/// 开关（现代化 toggle）
pub fn toggle(ui: &mut egui::Ui, value: &mut bool, pal: &Palette) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(36.0, 20.0), Sense::click());
    let anim = ui.ctx().animate_bool_with_time(resp.id, *value, 0.12);
    let on = anim > 0.5;
    let track = if on { pal.accent } else { pal.bg_selected };
    let knob_x = rect.left() + 2.0 + anim * (rect.width() - 24.0);
    ui.painter().rect_filled(rect, CornerRadius::same(10), track);
    ui.painter().circle_filled(
        egui::pos2(knob_x + 10.0, rect.center().y),
        8.0,
        if on { Color32::WHITE } else { pal.text_secondary },
    );
    if resp.clicked() {
        *value = !*value;
    }
    resp
}

/// 区块标题（侧边栏）
pub fn section_header(ui: &mut egui::Ui, title: &str, pal: &Palette) {
    ui.add_space(2.0);
    ui.label(
        RichText::new(title)
            .size(11.0)
            .color(pal.text_disabled)
            .strong(),
    );
    ui.add_space(2.0);
}

/// 卡片（欢迎页 / 书库）
#[allow(dead_code)]
pub fn card(
    ui: &mut egui::Ui,
    title: &str,
    subtitle: &str,
    icon: &str,
    pal: &Palette,
) -> egui::Response {
    let width = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, 64.0), Sense::click());
    let hover = resp.hovered();
    let bg = if hover { pal.bg_hover } else { pal.bg_panel_alt };
    ui.painter().rect_filled(rect, CornerRadius::same(10), bg);
    if hover {
        ui.painter()
            .rect_stroke(rect, CornerRadius::same(10), Stroke::new(1.0, pal.accent.gamma_multiply(0.7)), egui::StrokeKind::Middle);
    } else {
        ui.painter()
            .rect_stroke(rect, CornerRadius::same(10), Stroke::new(1.0, pal.border), egui::StrokeKind::Middle);
    }
    ui.painter().text(
        egui::pos2(rect.left() + 16.0, rect.center().y),
        Align2::LEFT_CENTER,
        icon,
        egui::FontId::proportional(20.0),
        pal.accent,
    );
    ui.painter().text(
        egui::pos2(rect.left() + 50.0, rect.top() + 19.0),
        Align2::LEFT_CENTER,
        title,
        egui::FontId::proportional(14.5),
        pal.text,
    );
    ui.painter().text(
        egui::pos2(rect.left() + 50.0, rect.top() + 42.0),
        Align2::LEFT_CENTER,
        subtitle,
        egui::FontId::proportional(11.5),
        pal.text_secondary,
    );
    resp
}

/// 空状态提示
pub fn empty_state(ui: &mut egui::Ui, icon: &str, title: &str, sub: &str, pal: &Palette) {
    let (rect, _) = ui.allocate_exact_size(ui.available_size(), Sense::hover());
    let center = rect.center();
    ui.painter().text(
        egui::pos2(center.x, center.y - 40.0),
        Align2::CENTER_CENTER,
        icon,
        egui::FontId::proportional(34.0),
        pal.text_disabled,
    );
    ui.painter().text(
        egui::pos2(center.x, center.y),
        Align2::CENTER_CENTER,
        title,
        egui::FontId::proportional(15.0),
        pal.text_secondary,
    );
    if !sub.is_empty() {
        ui.painter().text(
            egui::pos2(center.x, center.y + 24.0),
            Align2::CENTER_CENTER,
            sub,
            egui::FontId::proportional(12.0),
            pal.text_disabled,
        );
    }
}

/// 输入框（统一风格）
pub fn text_input(
    ui: &mut egui::Ui,
    value: &mut String,
    hint: &str,
    pal: &Palette,
) -> egui::Response {
    ui.add(
        egui::TextEdit::singleline(value)
            .hint_text(RichText::new(hint).color(pal.text_disabled))
            .text_color(pal.text)
            .desired_width(f32::INFINITY)
            .margin(Margin::symmetric(10, 6))
            
            .background_color(pal.bg_panel_alt),
    )
}

/// 多行输入框
pub fn text_area(
    ui: &mut egui::Ui,
    value: &mut String,
    hint: &str,
    rows: usize,
    pal: &Palette,
) -> egui::Response {
    ui.add(
        egui::TextEdit::multiline(value)
            .hint_text(RichText::new(hint).color(pal.text_disabled))
            .text_color(pal.text)
            .desired_width(f32::INFINITY)
            .desired_rows(rows)
            .margin(Margin::symmetric(10, 6))
            
            .background_color(pal.bg_panel_alt),
    )
}

/// 模态对话框：返回 None 时窗口已关闭
pub fn modal(
    ctx: &egui::Context,
    id: &str,
    title: &str,
    size: Vec2,
    pal: &Palette,
    content: impl FnOnce(&mut egui::Ui),
) -> Option<()> {
    // 背景遮罩
    let overlay_id = Id::new(format!("{}_overlay", id));
    let mut open = true;
    let screen = ctx.content_rect();
    egui::Area::new(overlay_id)
        .order(egui::Order::Tooltip)
        .fixed_pos(screen.min)
        .interactable(true)
        .show(ctx, |ui| {
            let (rect, resp) = ui.allocate_exact_size(screen.size(), Sense::click());
            ui.painter().rect_filled(rect, 0.0, Color32::from_black_alpha(120));
            if resp.clicked() {
                open = false;
            }
        });

    if !open {
        return None;
    }

    let frame = Frame::new()
        .fill(pal.bg_elevated)
        .stroke(Stroke::new(1.0, pal.border))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::same(18))
        .shadow(egui::Shadow {
            offset: [0, 8],
            blur: 32,
            spread: 0,
            color: Color32::from_black_alpha(140),
        });

    let area = egui::Area::new(Id::new(id))
        .order(egui::Order::Foreground)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .constrain(true);
    let mut closed = false;
    area.show(ctx, |ui| {
        frame.show(ui, |ui| {
            ui.set_min_size(size);
            ui.set_width(size.x - 36.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new(title).size(16.0).strong().color(pal.text));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if icon_btn(ui, IC_CLOSE, "关闭", pal).clicked() {
                        closed = true;
                    }
                });
            });
            ui.add_space(10.0);
            content(ui);
        });
    });
    if closed {
        None
    } else {
        Some(())
    }
}

/// 弹窗阴影边框的通用 Frame
pub fn popup_frame(pal: &Palette) -> Frame {
    Frame::new()
        .fill(pal.bg_elevated)
        .stroke(Stroke::new(1.0, pal.border))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::same(6))
        .shadow(egui::Shadow {
            offset: [0, 6],
            blur: 24,
            spread: 0,
            color: Color32::from_black_alpha(120),
        })
}

/// 竖向分隔线
#[allow(dead_code)]
pub fn v_sep(ui: &mut egui::Ui, pal: &Palette) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(1.0, ui.available_height()), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, pal.border);
}

/// 横向分隔线
pub fn h_sep(ui: &mut egui::Ui, pal: &Palette) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, pal.border);
}
