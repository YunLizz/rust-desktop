//! 导出：txt / md（导出为通用文本格式，无加密）

use crate::model::Novel;
use std::path::Path;

/// 导出纯文本
pub fn export_txt(novel: &Novel, chapters: &[(String, String, String)], path: &Path) -> Result<(), String> {
    let mut out = String::new();
    out.push_str(&format!("《{}》\n", novel.meta.title));
    if !novel.meta.author.is_empty() {
        out.push_str(&format!("作者：{}\n", novel.meta.author));
    }
    if !novel.meta.description.is_empty() {
        out.push_str(&format!("简介：{}\n", novel.meta.description));
    }
    out.push('\n');
    for vol in &novel.volumes {
        let vol_chapters: Vec<_> = chapters.iter().filter(|(id, _, _)| vol.chapters.iter().any(|c| &c.id == id)).collect();
        if vol_chapters.is_empty() {
            continue;
        }
        if vol.title != "正文" || vol.chapters.len() > 0 {
            out.push_str(&format!("\n{}\n{}\n\n", "=".repeat(20), vol.title));
        }
        for (_, title, text) in vol_chapters {
            out.push_str(&format!("\n{}\n{}\n\n", title, "-".repeat(12)));
            out.push_str(text);
            if !text.ends_with('\n') {
                out.push('\n');
            }
        }
    }
    std::fs::write(path, out).map_err(|e| format!("写入文件失败: {}", e))
}

/// 导出 Markdown
pub fn export_md(novel: &Novel, chapters: &[(String, String, String)], path: &Path) -> Result<(), String> {
    let mut out = String::new();
    out.push_str(&format!("# {}\n\n", novel.meta.title));
    if !novel.meta.author.is_empty() {
        out.push_str(&format!("> 作者：{}\n\n", novel.meta.author));
    }
    if !novel.meta.description.is_empty() {
        out.push_str(&format!("{}\n\n---\n", novel.meta.description));
    }
    for vol in &novel.volumes {
        let vol_chapters: Vec<_> = chapters.iter().filter(|(id, _, _)| vol.chapters.iter().any(|c| &c.id == id)).collect();
        if vol_chapters.is_empty() {
            continue;
        }
        if vol.title != "正文" {
            out.push_str(&format!("\n## {}\n\n", vol.title));
        }
        for (_, title, text) in vol_chapters {
            out.push_str(&format!("\n### {}\n\n", title));
            out.push_str(text);
            if !text.ends_with('\n') {
                out.push('\n');
            }
        }
    }
    std::fs::write(path, out).map_err(|e| format!("写入文件失败: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Novel;

    #[test]
    fn export_formats() {
        let mut n = Novel::new("导出测试", "我", "简介", "仙侠");
        n.add_volume("卷一");
        let vid = n.volumes[0].id.clone();
        let cid = n.add_chapter(Some(&vid), "第一章 开始");
        let chapters = vec![(cid.clone(), "第一章 开始".to_string(), "正文内容第一行。".to_string())];
        let dir = std::env::temp_dir().join("jinshu_test");
        std::fs::create_dir_all(&dir).unwrap();
        let md = dir.join("t.md");
        export_md(&n, &chapters, &md).unwrap();
        let md_text = std::fs::read_to_string(&md).unwrap();
        assert!(md_text.contains("# 导出测试"));
        assert!(md_text.contains("### 第一章 开始"));
        assert!(md_text.contains("正文内容第一行。"));
        let txt = dir.join("t.txt");
        export_txt(&n, &chapters, &txt).unwrap();
        let txt_text = std::fs::read_to_string(&txt).unwrap();
        assert!(txt_text.contains("《导出测试》"));
        assert!(txt_text.contains("第一章 开始"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
