use eframe::egui::{
    Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Stroke, TextStyle,
};

// 内置兜底 CJK 字体，彻底解决打包后、用户单独拷贝 exe、或系统无中文字体时的字形缺失。
// include_bytes! 在编译时直接将字体文件的字节流嵌入二进制。
// 即使外部字体（assets/fonts 下或系统字体）可用，外部字体仍会优先使用（find_cjk_font 的返回优先级高于内嵌）。
const BUILTIN_FONT_SANS: &[u8] = include_bytes!("../assets/fonts/NotoSansSC-VF.ttf");
const BUILTIN_FONT_SERIF: &[u8] = include_bytes!("../assets/fonts/NotoSerifSC-VF.ttf");

#[derive(Clone, Copy, PartialEq)]
pub struct Palette {
    pub dark: bool,
    pub bg_chrome: Color32,   // 标题栏 / 活动栏 / 状态栏
    pub bg_panel: Color32,    // 侧边栏面板
    pub bg_panel_alt: Color32, // 树形列表悬浮行
    pub bg_editor: Color32,   // 编辑器
    pub bg_elevated: Color32, // 弹层 / 命令面板
    pub bg_hover: Color32,
    pub bg_selected: Color32,
    pub border: Color32,
    pub text: Color32,
    pub text_secondary: Color32,
    pub text_disabled: Color32,
    pub accent: Color32,
    pub accent_dim: Color32,
    pub danger: Color32,
    pub warn: Color32,
    pub ok: Color32,
    pub blue: Color32,
    pub purple: Color32,
    pub cyan: Color32,
    pub pink: Color32,
}

pub const ACCENT_PRESETS: [(&str, [u8; 3]); 8] = [
    ("晨曦蓝", [0x4C, 0x8D, 0xFF]),
    ("星云紫", [0xA8, 0x82, 0xFF]),
    ("松石青", [0x3F, 0xC8, 0xC8]),
    ("鎏金橙", [0xE9, 0x97, 0x3F]),
    ("竹叶绿", [0x3D, 0xBE, 0x6E]),
    ("绯红", [0xFA, 0x6E, 0x9C]),
    ("黛紫蓝", [0x6E, 0x8B, 0xFF]),
    ("樱粉", [0xF2, 0x8B, 0xC2]),
];

impl Palette {
    pub fn dark(accent: [u8; 3]) -> Self {
        let accent = Color32::from_rgb(accent[0], accent[1], accent[2]);
        Self {
            dark: true,
            bg_chrome: Color32::from_rgb(0x19, 0x19, 0x1E),
            bg_panel: Color32::from_rgb(0x20, 0x20, 0x26),
            bg_panel_alt: Color32::from_rgb(0x25, 0x25, 0x2C),
            bg_editor: Color32::from_rgb(0x1C, 0x1C, 0x22),
            bg_elevated: Color32::from_rgb(0x28, 0x28, 0x30),
            bg_hover: Color32::from_rgb(0x2E, 0x2E, 0x37),
            bg_selected: Color32::from_rgb(0x35, 0x35, 0x40),
            border: Color32::from_rgb(0x2B, 0x2B, 0x33),
            text: Color32::from_rgb(0xDC, 0xE0, 0xE5),
            text_secondary: Color32::from_rgb(0x9B, 0xA0, 0xA8),
            text_disabled: Color32::from_rgb(0x5F, 0x63, 0x6B),
            accent,
            accent_dim: accent.gamma_multiply(0.55),
            danger: Color32::from_rgb(0xFB, 0x46, 0x4C),
            warn: Color32::from_rgb(0xE9, 0x97, 0x3F),
            ok: Color32::from_rgb(0x44, 0xCF, 0x6E),
            blue: Color32::from_rgb(0x4C, 0x8D, 0xFF),
            purple: Color32::from_rgb(0xA8, 0x82, 0xFF),
            cyan: Color32::from_rgb(0x53, 0xDF, 0xDD),
            pink: Color32::from_rgb(0xFA, 0x6E, 0x9C),
        }
    }

    pub fn light(accent: [u8; 3]) -> Self {
        let accent = Color32::from_rgb(accent[0], accent[1], accent[2]);
        Self {
            dark: false,
            bg_chrome: Color32::from_rgb(0xF3, 0xF3, 0xF5),
            bg_panel: Color32::from_rgb(0xFA, 0xFA, 0xFC),
            bg_panel_alt: Color32::from_rgb(0xF0, 0xF0, 0xF4),
            bg_editor: Color32::from_rgb(0xFF, 0xFF, 0xFF),
            bg_elevated: Color32::from_rgb(0xFF, 0xFF, 0xFF),
            bg_hover: Color32::from_rgb(0xE9, 0xE9, 0xEF),
            bg_selected: Color32::from_rgb(0xDD, 0xDD, 0xE8),
            border: Color32::from_rgb(0xE2, 0xE2, 0xE8),
            text: Color32::from_rgb(0x26, 0x26, 0x2E),
            text_secondary: Color32::from_rgb(0x6B, 0x6E, 0x78),
            text_disabled: Color32::from_rgb(0xAE, 0xB1, 0xBA),
            accent,
            accent_dim: accent.gamma_multiply(0.6),
            danger: Color32::from_rgb(0xD1, 0x38, 0x3E),
            warn: Color32::from_rgb(0xC4, 0x7E, 0x1F),
            ok: Color32::from_rgb(0x2F, 0x9E, 0x5B),
            blue: Color32::from_rgb(0x2F, 0x6F, 0xE0),
            purple: Color32::from_rgb(0x8B, 0x5C, 0xF6),
            cyan: Color32::from_rgb(0x0E, 0x9F, 0x9E),
            pink: Color32::from_rgb(0xD6, 0x3A, 0x8C),
        }
    }
}

/// 查找可用的 CJK 字体文件（安装目录 assets 优先，其次系统字体）
fn find_cjk_font(serif: bool) -> Option<std::borrow::Cow<'static, [u8]>> {
    use std::path::PathBuf;

    // 1. 环境变量显式指定
    let env_key = if serif { "JINSHU_FONT_SERIF" } else { "JINSHU_FONT_SANS" };
    if let Ok(p) = std::env::var(env_key) {
        if let Ok(bytes) = std::fs::read(&p) {
            return Some(std::borrow::Cow::Owned(bytes));
        }
    }

    // ---- 1.5 硬优先：Windows 系统自带官方 CJK 字体（msyh / simsun 是 egui 可解析的
    //      非 VF 静态 TTC/TTF）。当前项目内嵌的 NotoSansSC-VF.ttf 为「可变字体」，
    //      在 egui 0.36 的 fontdue/ab_glyph 解析链条下会导致字形查找失败：
    //      表现为"ASCII 正常、中文全空白"，更糟的是把它放 families[0] 时会导致
    //      "任何文字都不显示"。因此先尝试系统自带字体，一定可用。 ----
    #[cfg(windows)]
    {
        let sys_fonts = PathBuf::from(r"C:\Windows\Fonts");
        let candidates: &[&str] = if serif {
            &["simsun.ttc", "nsimsun.ttf", "simsun.ttf"]
        } else {
            &["msyh.ttc", "msyh.ttf", "msyhbd.ttc", "msyhl.ttc", "simhei.ttf"]
        };
        for name in candidates {
            let p = sys_fonts.join(name);
            if let Ok(bytes) = std::fs::read(&p) {
                return Some(std::borrow::Cow::Owned(bytes));
            }
        }
        // 用户级字体目录（有些机器字体装在这里）
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            let user_fonts = PathBuf::from(local).join("Microsoft").join("Windows").join("Fonts");
            for name in candidates {
                let p = user_fonts.join(name);
                if let Ok(bytes) = std::fs::read(&p) {
                    return Some(std::borrow::Cow::Owned(bytes));
                }
            }
        }
    }

    // 2. 候选搜索目录（优先级递减）
    let mut dirs: Vec<PathBuf> = Vec::new();
    // 2a. 可执行文件同级目录（打包后场景）：<exe>/assets/fonts
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            dirs.push(parent.join("assets").join("fonts"));
        }
    }
    // 2b. 当前工作目录（cargo run 场景，或用户在项目根目录通过终端启动）
    if let Ok(cwd) = std::env::current_dir() {
        dirs.push(cwd.join("assets").join("fonts"));
    }
    // 2c. 开发模式：CARGO_MANIFEST_DIR 指向 Cargo.toml 所在目录
    if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        dirs.push(PathBuf::from(manifest).join("assets").join("fonts"));
    }
    // 2d. Windows 系统字体目录（兜底：扫目录名匹配）
    #[cfg(windows)]
    {
        dirs.push(PathBuf::from(r"C:\Windows\Fonts"));
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            dirs.push(PathBuf::from(local).join("Microsoft").join("Windows").join("Fonts"));
        }
    }
    // 2e. Linux 系统字体目录
    let system_dirs: &[&str] = &[
        "/usr/share/fonts/noto-cjk",
        "/usr/share/fonts/opentype/noto",
        "/usr/share/fonts/opentype/source-han-serif",
        "/usr/share/fonts/opentype/source-han-sans",
        "/usr/share/fonts/truetype/wqy",
        "/usr/share/fonts/wqy-zenhei",
        "/usr/share/fonts/wqy-microhei",
        "/usr/share/fonts/adobe-source-han-sans",
        "/usr/share/fonts/adobe-source-han-serif",
        "/usr/local/share/fonts",
    ];
    for d in system_dirs {
        dirs.push(PathBuf::from(d));
    }

    let names: &[&str] = if serif {
        &[
            "notoserifsc", "noto_serif_sc", "NotoSerifSC", "NotoSerifCJK", "SourceHanSerifSC",
            "simsun", "nsimsun", "sung", "宋体",
        ]
    } else {
        &[
            "notosanssc", "noto_sans_sc", "NotoSansSC", "NotoSansCJK", "SourceHanSansSC", "msyh",
            "simhei", "wqy", "yahei", "hei",
        ]
    };

    for dir in dirs {
        if let Ok(rd) = std::fs::read_dir(&dir) {
            let mut files: Vec<_> = rd.flatten().map(|e| e.path()).collect();
            files.sort();
            for p in files {
                let fname = p
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                if !(fname.ends_with(".ttf") || fname.ends_with(".otf") || fname.ends_with(".ttc")) {
                    continue;
                }
                if names.iter().any(|n| fname.contains(&n.to_lowercase())) {
                    if let Ok(bytes) = std::fs::read(&p) {
                        return Some(std::borrow::Cow::Owned(bytes));
                    }
                }
            }
        }
    }
    // 终极兜底：返回编译时内嵌的字体字节。
    // 注意：当前内嵌（NotoSansSC-VF.ttf / NotoSerifSC-VF.ttf）是「可变字体」，
    //       egui 0.36 在部分环境下对 VF 字形查找会异常。
    //       只有在系统/目录扫描都找不到任何 CJK 字体时才启用此兜底。
    if serif {
        Some(std::borrow::Cow::Borrowed(BUILTIN_FONT_SERIF))
    } else {
        Some(std::borrow::Cow::Borrowed(BUILTIN_FONT_SANS))
    }
}

/// 安装字体与文本样式。editor_serif 控制编辑区使用宋体还是黑体。
pub fn install_fonts(ctx: &egui::Context, editor_serif: bool) {
    let sans = find_cjk_font(false);
    let serif = find_cjk_font(true);

    let mut fonts = FontDefinitions::default();

    // ---- 关键修复：CJK 字体必须放在 Proportional / Monospace 族的最前列（insert(0)）。
    //      egui 按 families 数组顺序逐字体查找字形：如果把 Ubuntu/Hack 放在前面，
    //      它们不含 CJK 字形，而 egui 0.36 的字体 fallback 在某些字体组合下会
    //      直接返回"字形缺失"（尤其是 Variable Font + 系统默认字体共存时），
    //      导致 CJK 字符虽然存在于后序字体里，但永远不会被命中，表现为"ASCII
    //      正常但中文全空白"。把 CJK 字体插在第一位即可彻底解决。 ----
    if let Some(sans) = sans {
        fonts.font_data.insert("cjk_sans".into(), std::sync::Arc::new(FontData::from_owned(sans.into_owned())));
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, "cjk_sans".into());
        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .insert(0, "cjk_sans".into());
    }
    if let Some(serif) = serif {
        fonts.font_data.insert("cjk_serif".into(), std::sync::Arc::new(FontData::from_owned(serif.into_owned())));
        let fam = if editor_serif { FontFamily::Monospace } else { FontFamily::Proportional };
        // 编辑区字体的优先顺序由 fam 决定：如果用户选"宋体"，则把 serif 放在 monospace 最前
        let target = fonts.families.entry(fam).or_default();
        target.insert(0, "cjk_serif".into());
        // 另一族也放一份 serif 在前列（但排在 sans 之后，作为 CJK 字形的兜底）
        let other = fonts.families.entry(if editor_serif { FontFamily::Proportional } else { FontFamily::Monospace }).or_default();
        // 在 cjk_sans 之后、系统字体之前插入 serif 兜底
        if let Some(pos) = other.iter().position(|f| f == "cjk_sans") {
            other.insert(pos + 1, "cjk_serif".into());
        } else {
            other.insert(0, "cjk_serif".into());
        }
    }
    ctx.set_fonts(fonts);
}

pub fn apply_visuals(ctx: &egui::Context, pal: &Palette, ui_scale: f32, base_font_size: f32) {
    ctx.set_pixels_per_point(ui_scale);
    // 关闭 egui 内置的 Ctrl+加号/减号 界面缩放，避免与编辑区字号快捷键冲突
    ctx.options_mut(|o| o.zoom_with_keyboard = false);

    // ---- egui 0.31 ~ 0.36 在 Windows 存在已知 bug：在 app_creator 或通过
    //      set_style_of + set_theme 设置的 text_styles / visuals 会被底层重置。
    //      官方 workaround：在每帧 update() 内通过 style_mut_of() / set_visuals()
    //      直接修改当前激活样式。见：https://github.com/emilk/egui/issues/5840 ----
    let theme = if pal.dark { egui::Theme::Dark } else { egui::Theme::Light };
    ctx.set_theme(theme);

    // 通过 style_mut_of 直接修改当前激活主题对应的 style，避免 set_style_of + set_theme
    // 组合被 egui 每帧开始时的默认覆盖。
    ctx.style_mut_of(theme, |style| {
        // 核弹级兜底：直接对所有控件强制指定默认 FontId，避免 TextStyle 映射命中默认极小字号
        style.override_font_id = Some(FontId::new(base_font_size, FontFamily::Proportional));
        style.override_text_style = Some(TextStyle::Body);

        style.spacing.item_spacing = egui::vec2(8.0, 5.0);
        style.spacing.button_padding = egui::vec2(10.0, 5.0);
        style.spacing.icon_spacing = 6.0;
        style.spacing.indent = 16.0;
        style.spacing.interact_size.y = 24.0;

        // 先清空旧映射，避免有未定义 TextStyle 命中极小的默认字号
        style.text_styles.clear();
        style.text_styles.insert(
            TextStyle::Heading,
            FontId::new(base_font_size + 6.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Body,
            FontId::new(base_font_size, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Button,
            FontId::new(base_font_size - 1.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Small,
            FontId::new(base_font_size - 3.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Monospace,
            FontId::new(base_font_size - 1.0, FontFamily::Monospace),
        );
        // 兜底：若 egui 后续版本新增 TextStyle::Name(_) 变体，再显式塞两个常用名字，
        // 避免命中 ~9px 的默认极小字号。
        if !style.text_styles.contains_key(&TextStyle::Name("heading".into())) {
            style.text_styles.insert(
                TextStyle::Name("heading".into()),
                FontId::new(base_font_size + 6.0, FontFamily::Proportional),
            );
        }
        if !style.text_styles.contains_key(&TextStyle::Name("monospace".into())) {
            style.text_styles.insert(
                TextStyle::Name("monospace".into()),
                FontId::new(base_font_size - 1.0, FontFamily::Monospace),
            );
        }
    });

    // ---- Visuals（颜色/圆角/阴影）每帧直接 set_visuals，保证 Windows 平台不被重置 ----
    let mut visuals = if pal.dark { egui::Visuals::dark() } else { egui::Visuals::light() };
    visuals.panel_fill = pal.bg_panel;
    visuals.window_fill = pal.bg_elevated;
    visuals.window_stroke = Stroke::new(1.0, pal.border);
    visuals.window_corner_radius = CornerRadius::same(10);
    visuals.menu_corner_radius = CornerRadius::same(8);
    visuals.popup_shadow = egui::Shadow {
        offset: [0, 6],
        blur: 24,
        spread: 0,
        color: Color32::from_black_alpha(90),
    };
    visuals.window_shadow = visuals.popup_shadow;
    visuals.extreme_bg_color = pal.bg_elevated;
    visuals.faint_bg_color = pal.bg_hover;
    visuals.code_bg_color = pal.bg_panel_alt;
    visuals.hyperlink_color = pal.accent;
    visuals.selection.bg_fill = pal.accent.gamma_multiply(0.35);
    visuals.selection.stroke = Stroke::new(1.0, pal.accent);
    visuals.text_cursor.stroke = Stroke::new(2.0, pal.accent);
    visuals.warn_fg_color = pal.warn;
    visuals.error_fg_color = pal.danger;

    let w = |fill: Color32| egui::style::WidgetVisuals {
        bg_fill: fill,
        weak_bg_fill: fill,
        bg_stroke: Stroke::new(1.0, pal.border),
        corner_radius: CornerRadius::same(6),
        fg_stroke: Stroke::new(1.0, pal.text),
        expansion: 0.0,
    };
    visuals.widgets.noninteractive = w(pal.bg_panel_alt);
    visuals.widgets.inactive = w(pal.bg_panel_alt);
    visuals.widgets.hovered = w(pal.bg_hover);
    visuals.widgets.active = w(pal.bg_selected);
    visuals.widgets.open = w(pal.bg_selected);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, pal.text_secondary);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, pal.text);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, pal.accent);

    ctx.set_visuals(visuals);
}

/// 主题色文字（用于状态栏等）
pub fn text_color_contrast(bg: Color32) -> Color32 {
    let lum = 0.299 * bg.r() as f32 + 0.587 * bg.g() as f32 + 0.114 * bg.b() as f32;
    if lum > 150.0 { Color32::from_rgb(30, 30, 30) } else { Color32::from_rgb(240, 240, 245) }
}
