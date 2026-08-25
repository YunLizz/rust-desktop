use chrono::Local;
use uuid::Uuid;

pub fn new_id() -> String {
    Uuid::new_v4().simple().to_string()
}

pub fn today() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

pub fn days_ago(n: i64) -> String {
    let d = Local::now().date_naive() - chrono::Duration::days(n);
    d.format("%Y-%m-%d").to_string()
}

pub fn now_ts() -> i64 {
    Local::now().timestamp()
}

pub fn format_ts(ts: i64) -> String {
    let dt = chrono::DateTime::from_timestamp(ts, 0);
    match dt {
        Some(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
        None => "-".to_string(),
    }
}

fn is_cjk(c: char) -> bool {
    matches!(c as u32,
        0x4E00..=0x9FFF | 0x3400..=0x4DBF | 0x20000..=0x2A6DF
        | 0xF900..=0xFAFF | 0x3000..=0x303F | 0xFF00..=0xFFEF
        | 0x3040..=0x30FF | 0x31F0..=0x31FF)
}

/// 字数统计：汉字与中文标点按字符计，连续拉丁字母/数字按一个词计
pub fn count_words(s: &str) -> u64 {
    let mut n: u64 = 0;
    let mut in_word = false;
    for ch in s.chars() {
        if is_cjk(ch) {
            n += 1;
            in_word = false;
        } else if ch.is_alphanumeric() {
            if !in_word {
                n += 1;
                in_word = true;
            }
        } else {
            in_word = false;
        }
    }
    n
}

/// 字符总数（不含空白）
pub fn count_chars(s: &str) -> u64 {
    s.chars().filter(|c| !c.is_whitespace()).count() as u64
}

/// 截断字符串到指定字符数，用于 AI 上下文裁剪
pub fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max).collect();
        out.push_str("\n\n……（已截断）");
        out
    }
}

/// 取末尾若干字符
pub fn tail_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        s.chars().rev().take(max).collect::<String>().chars().rev().collect()
    }
}

/// 用全角/半角空格填充首行缩进（中文写作习惯：两字符缩进）
pub fn indent_two(s: &str) -> String {
    format!("　　{}", s)
}
