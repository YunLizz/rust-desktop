//! 自研小说编辑器组件：基于 egui TextEdit + 自定义 layouter
//! - 同步行号栏（逻辑行，自动换行下不错位）
//! - 可选的 Markdown 高亮（默认关闭，符合中文写作习惯）
//! - 行距 / 字距 / 字体 / 字号控制
//! - 查找结果跳转（通过 TextEditState 设置光标）

use eframe::egui::{
    self, text::{CCursor, CCursorRange, CharIndex, LayoutJob, TextFormat, TextWrapping},
    Align2, Color32, FontId, Galley, Rect, Sense, Stroke, TextBuffer, TextEdit, Vec2,
};

use crate::theme::Palette;

pub struct EditorStyle {
    pub font_id: FontId,
    pub line_spacing: f32,
    pub letter_spacing: f32,
    pub wrap: bool,
    pub md_highlight: bool,
    pub line_numbers: bool,
    pub gutter_bg: Color32,
    pub text_color: Color32,
    pub accent: Color32,
    pub hint_text: String,
}

pub struct EditorOutput {
    pub response: egui::Response,
    pub galley: std::sync::Arc<Galley>,
    pub cursor_range: Option<CCursorRange>,
    pub galley_pos: egui::Pos2,
    pub changed: bool,
}

/// 构建 Markdown 高亮排版任务
fn layout_markdown(text: &str, style: &EditorStyle, pal: &Palette, wrap_width: f32) -> LayoutJob {
    let line_h = style.font_id.size * style.line_spacing;
    let default_fmt = || TextFormat {
        font_id: style.font_id.clone(),
        color: style.text_color,
        extra_letter_spacing: style.letter_spacing,
        line_height: Some(line_h),
        ..Default::default()
    };

    let mut job = LayoutJob::default();
    job.break_on_newline = true;
    job.wrap = TextWrapping {
        max_width: if style.wrap { wrap_width } else { f32::INFINITY },
        max_rows: usize::MAX,
        break_anywhere: true,
        ..Default::default()
    };

    let heading_fmt = |_level: usize| TextFormat {
        font_id: style.font_id.clone(),
        color: pal.accent,
        extra_letter_spacing: style.letter_spacing,
        line_height: Some(line_h),
        ..Default::default()
    };
    let code_fmt = TextFormat {
        font_id: FontId::monospace(style.font_id.size - 1.0),
        color: pal.cyan,
        background: pal.bg_panel_alt,
        extra_letter_spacing: 0.0,
        line_height: Some(line_h),
        ..Default::default()
    };
    let quote_fmt = TextFormat {
        font_id: style.font_id.clone(),
        color: pal.text_secondary,
        italics: true,
        extra_letter_spacing: style.letter_spacing,
        line_height: Some(line_h),
        ..Default::default()
    };
    let link_fmt = TextFormat {
        font_id: style.font_id.clone(),
        color: pal.blue,
        underline: Stroke::new(1.0, pal.blue),
        extra_letter_spacing: style.letter_spacing,
        line_height: Some(line_h),
        ..Default::default()
    };
    let strong_fmt = TextFormat {
        font_id: style.font_id.clone(),
        color: pal.text,
        extra_letter_spacing: style.letter_spacing,
        line_height: Some(line_h),
        ..Default::default()
    };

    let mut i = 0usize;
    let chars: Vec<char> = text.chars().collect();
    while i < chars.len() {
        let rest: String = chars[i..].iter().collect();
        // 标题：#~###### 开头
        if rest.starts_with('\n') {
            job.append("\n", 0.0, default_fmt());
            i += 1;
            continue;
        }
        let line_start = i == 0 || chars[i - 1] == '\n';
        if line_start {
            let mut hashes = 0;
            while hashes < 6 && i + hashes < chars.len() && chars[i + hashes] == '#' {
                hashes += 1;
            }
            if hashes >= 1
                && i + hashes < chars.len()
                && (chars[i + hashes] == ' ' || chars[i + hashes] == '\t')
            {
                job.append(&format!("{} ", "#".repeat(hashes)), 0.0, heading_fmt(hashes));
                i += hashes + 1;
                continue;
            }
        }
        // 引用
        if line_start && chars[i] == '>' {
            job.append("> ", 0.0, quote_fmt.clone());
            i += 1;
            continue;
        }
        // 分隔线
        if line_start {
            let line_end = rest.find('\n').unwrap_or(rest.len());
            let line = &rest[..line_end];
            let trimmed = line.trim();
            if trimmed.chars().all(|c| c == '-' || c == '*') && trimmed.chars().count() >= 3 {
                job.append(line, 0.0, quote_fmt.clone());
                i += line.len();
                continue;
            }
        }
        // 行内标记
        let c = chars[i];
        if c == '`' {
            // 代码段
            if let Some(end) = rest[1..].find('`') {
                job.append(&rest[..=end + 1], 0.0, code_fmt.clone());
                i += end + 2;
                continue;
            }
        }
        if c == '[' {
            if let Some(end) = rest.find(']') {
                let label = &rest[1..end];
                if end + 1 < rest.len() && rest.as_bytes()[end + 1] == b'(' {
                    if let Some(url_end) = rest[end + 2..].find(')') {
                        job.append(label, 0.0, link_fmt.clone());
                        i += end + 2 + url_end + 1;
                        continue;
                    }
                }
            }
        }
        if c == '*' {
            let doubled = i + 1 < chars.len() && chars[i + 1] == '*';
            let mark_len = if doubled { 2 } else { 1 };
            let closer = if doubled { "**" } else { "*" };
            if let Some(end) = rest[mark_len..].find(closer) {
                job.append(&rest[..mark_len + end + mark_len], 0.0, if doubled { strong_fmt.clone() } else { quote_fmt.clone() });
                i += mark_len + end + mark_len;
                continue;
            }
        }
        // 普通字符
        job.append(&c.to_string(), 0.0, default_fmt());
        i += 1;
    }
    job
}

fn layout_plain(text: &str, style: &EditorStyle, wrap_width: f32) -> LayoutJob {
    let line_h = style.font_id.size * style.line_spacing;
    let fmt = TextFormat {
        font_id: style.font_id.clone(),
        color: style.text_color,
        extra_letter_spacing: style.letter_spacing,
        line_height: Some(line_h),
        ..Default::default()
    };
    let mut job = LayoutJob::default();
    job.break_on_newline = true;
    job.wrap = TextWrapping {
        max_width: if style.wrap { wrap_width } else { f32::INFINITY },
        max_rows: usize::MAX,
        break_anywhere: true,
        ..Default::default()
    };
    job.append(text, 0.0, fmt);
    job
}

/// 设置编辑器光标位置（用于查找跳转）
pub fn set_cursor(ctx: &egui::Context, state_id: egui::Id, char_index: usize) {
    if let Some(mut st) = TextEdit::load_state(ctx, state_id) {
        let cc = CCursor {
            index: CharIndex(char_index),
            prefer_next_row: false,
        };
        st.cursor.set_char_range(Some(CCursorRange::one(cc)));
        st.store(ctx, state_id);
    }
}

/// 显示编辑器。state_id_salt 用于区分不同标签页的编辑状态。
pub fn show_editor(
    ui: &mut egui::Ui,
    state_id_salt: &str,
    text: &mut String,
    style: &EditorStyle,
    pal: &Palette,
) -> EditorOutput {
    let font_id = style.font_id.clone();
    let row_height = ui.fonts_mut(|f| f.row_height(&font_id));
    let extra = (style.line_spacing - 1.0) * row_height;
    ui.style_mut().spacing.extra_text_line_spacing = extra;

    let _editor_id = ui.make_persistent_id(state_id_salt);
    let rows_key = ui.make_persistent_id((state_id_salt, "rows"));
    let cursor_key = ui.make_persistent_id((state_id_salt, "cursor"));
    let cached_rows = ui.data_mut(|d| d.get_persisted::<usize>(rows_key)).unwrap_or(24);

    let hint = style.hint_text.clone();
    let md = style.md_highlight;

    let mut result: Option<(egui::Response, std::sync::Arc<Galley>, Option<CCursorRange>, egui::Pos2, bool)> = None;
    let mut cursor_changed = false;

    let gutter_w = if style.line_numbers {
        let digits = text.lines().count().max(1).to_string().len().max(3) as f32;
        row_height * 1.35 + digits * ui.fonts_mut(|f| f.glyph_width(&FontId::monospace(style.font_id.size - 2.0), '8')) + 10.0
    } else {
        0.0
    };

    egui::ScrollArea::vertical()
        .id_salt(("jinshu_scroll", state_id_salt))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.horizontal_top(|ui| {
                if style.line_numbers {
                    ui.allocate_exact_size(Vec2::new(gutter_w, 10.0), Sense::hover());
                }
                let mut layouter = |ui: &egui::Ui, buf: &dyn TextBuffer, wrap_width: f32| {
                    let s = buf.as_str();
                    let job = if md {
                        layout_markdown(s, style, pal, wrap_width)
                    } else {
                        layout_plain(s, style, wrap_width)
                    };
                    ui.fonts_mut(|f| f.layout_job(job))
                };
                let mut te = TextEdit::multiline(text)
                    .id_source(state_id_salt)
                    .desired_rows(cached_rows.max(1))
                    .desired_width(f32::INFINITY)
                    .text_color(style.text_color)
                    .layouter(&mut layouter);
                if !hint.is_empty() {
                    te = te.hint_text(hint);
                }
                let out = te.show(ui);
                let changed = out.response.changed();
                let galley = out.galley.clone();
                let cursor_range = out.cursor_range;
                let galley_pos = out.galley_pos;

                // 光标移动时跟随滚动
                let focused = out.response.has_focus();
                if focused {
                    let last_cursor = ui.data_mut(|d| d.get_persisted::<usize>(cursor_key));
                    if let Some(cr) = cursor_range {
                        let cur = cr.primary.index.0;
                        if last_cursor != Some(cur) {
                            ui.data_mut(|d| d.insert_persisted(cursor_key, cur));
                            cursor_changed = true;
                        }
                    }
                }
                result = Some((out.response.response, galley, cursor_range, galley_pos, changed));
            });
        });

    let (resp, galley, cursor_range, galley_pos, changed) = result.expect("editor always shows");

    // 更新行高缓存（保证 TextEdit 高度与 galley 一致，行号不错位）
    let rows = galley.rows.len();
    if cached_rows != rows {
        ui.data_mut(|d| d.insert_persisted(rows_key, rows));
    }

    // 光标跟随滚动（在滚动区内调用，但此处用外层 ui 的 scroll_to_rect 亦可，
    // 因为 scroll 目标是当前滚动区）
    if cursor_changed {
        if let Some(cr) = cursor_range {
            let rect = galley.pos_from_cursor(cr.primary).translate(galley_pos.to_vec2());
            ui.scroll_to_rect(rect.shrink(20.0), None);
        }
    }

    // 当前行高亮 + 行号栏
    if style.line_numbers {
        let text_rect = resp.rect;
        let gutter = Rect::from_min_size(
            egui::pos2(text_rect.left() - gutter_w, text_rect.top()),
            Vec2::new(gutter_w, text_rect.height()),
        );
        let painter = ui.painter();
        painter.rect_filled(gutter, 0.0, style.gutter_bg);
        painter.vline(
            gutter.right(),
            gutter.y_range(),
            Stroke::new(1.0, pal.border),
        );

        // 当前行高亮
        if let Some(cr) = cursor_range {
            let row_rect = galley.pos_from_cursor(cr.primary).translate(galley_pos.to_vec2());
            let hl = Rect::from_min_max(
                egui::pos2(text_rect.left(), row_rect.top()),
                egui::pos2(text_rect.right(), row_rect.bottom()),
            );
            painter.rect_filled(hl, 2.0, pal.accent.gamma_multiply(0.10));
        }

        // 行号（仅逻辑行首行绘制，自动换行的续行不重复编号）
        let num_font = FontId::monospace(style.font_id.size - 2.0);
        let mut y = galley_pos.y;
        let mut line_no: usize = 1;
        let mut new_line = true;
        for row in &galley.rows {
            let row_h = row.size.y;
            if new_line && row_h > 0.0 {
                let center = egui::pos2(gutter.right() - 10.0, y + row_h * 0.5);
                painter.text(
                    center,
                    Align2::RIGHT_CENTER,
                    line_no.to_string(),
                    num_font.clone(),
                    pal.text_secondary,
                );
            }
            if row.ends_with_newline {
                line_no += 1;
                new_line = true;
            } else {
                new_line = false;
            }
            y += row_h;
        }
    } else if let Some(cr) = cursor_range {
        // 无行号时仍画当前行高亮
        let row_rect = galley.pos_from_cursor(cr.primary).translate(galley_pos.to_vec2());
        let text_rect = resp.rect;
        let hl = Rect::from_min_max(
            egui::pos2(text_rect.left(), row_rect.top()),
            egui::pos2(text_rect.right(), row_rect.bottom()),
        );
        ui.painter().rect_filled(hl, 2.0, pal.accent.gamma_multiply(0.08));
    }

    EditorOutput {
        response: resp,
        galley,
        cursor_range,
        galley_pos,
        changed,
    }
}
