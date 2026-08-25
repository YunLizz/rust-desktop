pub mod crypto;

use crate::model::Novel;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const KEY_FILE: &str = ".jinshu_key";
const SETTINGS_FILE: &str = "settings.jsr";

#[derive(Serialize, Deserialize, Clone)]
pub struct AiSettings {
    pub protocol: String, // "openai" | "anthropic"
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub system_prompt: String,
    pub timeout_secs: u64,
    /// 自动注入设定（人物/地点出现时注入对应卡片）
    pub inject_lore: bool,
}

impl Default for AiSettings {
    fn default() -> Self {
        Self {
            protocol: "openai".into(),
            base_url: "https://api.openai.com/v1".into(),
            api_key: String::new(),
            model: "gpt-4o-mini".into(),
            temperature: 0.8,
            max_tokens: 4096,
            system_prompt: "你是一位资深的中文小说创作助手，擅长网文与严肃文学的叙事技巧。你的回答要具体、可落地、贴合小说上下文。".into(),
            timeout_secs: 180,
            inject_lore: true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EditorSettings {
    /// "sans" 黑体类 | "serif" 宋体类
    pub font: String,
    pub font_size: f32,
    pub line_spacing: f32,
    pub wrap: bool,
    /// 是否把 #、* 等渲染为 Markdown 高亮（中文写作默认关闭）
    pub markdown_highlight: bool,
    /// 保存时自动为首段添加全角缩进（若未手动缩进）
    pub auto_indent: bool,
    pub show_line_numbers: bool,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            font: "serif".into(),
            font_size: 17.0,
            line_spacing: 1.9,
            wrap: true,
            markdown_highlight: false,
            auto_indent: false,
            show_line_numbers: true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RecentNovel {
    pub id: String,
    pub title: String,
    pub opened_at: i64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub theme: String, // "dark" | "light"
    pub accent: [u8; 3],
    pub ui_scale: f32,
    pub editor: EditorSettings,
    pub ai: AiSettings,
    pub autosave_secs: u64,
    pub recent: Vec<RecentNovel>,
    pub sidebar_width: f32,
    pub ai_panel_width: f32,
    pub last_novel_id: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "dark".into(),
            accent: [0x6E, 0x8B, 0xFF],
            ui_scale: 1.0,
            editor: EditorSettings::default(),
            ai: AiSettings::default(),
            autosave_secs: 5,
            recent: Vec::new(),
            sidebar_width: 300.0,
            ai_panel_width: 340.0,
            last_novel_id: None,
        }
    }
}

#[derive(Clone)]
pub struct Store {
    pub data_dir: PathBuf,
    pub key: [u8; 32],
}

/// 数据目录解析：环境变量 JINSHU_DATA_DIR > 可执行文件同目录/data（安装目录缓存） > 用户目录兜底
pub fn resolve_data_dir() -> PathBuf {
    if let Ok(d) = std::env::var("JINSHU_DATA_DIR") {
        if !d.trim().is_empty() {
            let p = PathBuf::from(d.trim());
            if std::fs::create_dir_all(&p).is_ok() {
                return p;
            }
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let dir = parent.join("data");
            if dir.exists() || std::fs::create_dir_all(&dir).is_ok() {
                return dir;
            }
        }
    }
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    let dir = PathBuf::from(home).join(".jinshu").join("data");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

#[cfg(unix)]
fn tighten_permissions(p: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(p, std::fs::Permissions::from_mode(0o700));
}
#[cfg(not(unix))]
fn tighten_permissions(_p: &Path) {}

impl Store {
    pub fn init() -> Result<Store, String> {
        let data_dir = resolve_data_dir();
        std::fs::create_dir_all(&data_dir).map_err(|e| format!("无法创建数据目录 {}: {}", data_dir.display(), e))?;
        tighten_permissions(&data_dir);
        std::fs::create_dir_all(data_dir.join("novels")).map_err(|e| e.to_string())?;

        let key_path = data_dir.join(KEY_FILE);
        let key = if key_path.exists() {
            let bytes = std::fs::read(&key_path).map_err(|e| format!("读取密钥文件失败: {}", e))?;
            if bytes.len() != 32 {
                return Err("密钥文件损坏（长度不为 32 字节），请删除 data/.jinshu_key 后重试（注意：删除后旧数据将无法解密）".into());
            }
            let mut k = [0u8; 32];
            k.copy_from_slice(&bytes);
            k
        } else {
            let k = crypto::generate_key();
            std::fs::write(&key_path, k).map_err(|e| format!("写入密钥文件失败: {}", e))?;
            tighten_permissions(&key_path);
            k
        };
        Ok(Store { data_dir, key })
    }

    pub fn key_fingerprint(&self) -> String {
        crypto::fingerprint(&self.key)
    }

    // ---------- 设置 ----------
    pub fn load_settings(&self) -> AppSettings {
        let path = self.data_dir.join(SETTINGS_FILE);
        match crypto::decrypt_file(&path, &self.key) {
            Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
            Err(_) => AppSettings::default(),
        }
    }

    pub fn save_settings(&self, s: &AppSettings) -> Result<(), String> {
        let json = serde_json::to_vec(s).map_err(|e| e.to_string())?;
        crypto::encrypt_file(&self.data_dir.join(SETTINGS_FILE), &self.key, &json)
    }

    // ---------- 小说 ----------
    pub fn novel_dir(&self, id: &str) -> PathBuf {
        self.data_dir.join("novels").join(id)
    }

    pub fn list_novels(&self) -> Vec<crate::model::NovelMeta> {
        let dir = self.data_dir.join("novels");
        let mut out = Vec::new();
        if let Ok(rd) = std::fs::read_dir(&dir) {
            for entry in rd.flatten() {
                let p = entry.path();
                let meta_path = p.join("novel.jsr");
                if let Ok(bytes) = crypto::decrypt_file(&meta_path, &self.key) {
                    if let Ok(novel) = serde_json::from_slice::<Novel>(&bytes) {
                        out.push(novel.meta);
                    }
                }
            }
        }
        out.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        out
    }

    pub fn load_novel(&self, id: &str) -> Result<Novel, String> {
        let meta_path = self.novel_dir(id).join("novel.jsr");
        let bytes = crypto::decrypt_file(&meta_path, &self.key)?;
        serde_json::from_slice(&bytes).map_err(|e| format!("小说数据解析失败: {}", e))
    }

    pub fn save_novel(&self, novel: &Novel) -> Result<(), String> {
        let dir = self.novel_dir(&novel.meta.id);
        std::fs::create_dir_all(dir.join("chapters")).map_err(|e| e.to_string())?;
        let json = serde_json::to_vec(novel).map_err(|e| e.to_string())?;
        crypto::encrypt_file(&dir.join("novel.jsr"), &self.key, &json)
    }

    pub fn load_chapter(&self, novel_id: &str, cid: &str) -> Result<String, String> {
        let p = self.novel_dir(novel_id).join("chapters").join(format!("{}.jsr", cid));
        let bytes = crypto::decrypt_file(&p, &self.key)?;
        String::from_utf8(bytes).map_err(|e| format!("章节内容编码错误: {}", e))
    }

    pub fn save_chapter(&self, novel_id: &str, cid: &str, text: &str) -> Result<(), String> {
        let p = self.novel_dir(novel_id).join("chapters").join(format!("{}.jsr", cid));
        crypto::encrypt_file(&p, &self.key, text.as_bytes())
    }

    pub fn delete_chapter_file(&self, novel_id: &str, cid: &str) {
        let p = self.novel_dir(novel_id).join("chapters").join(format!("{}.jsr", cid));
        let _ = std::fs::remove_file(p);
    }

    pub fn delete_novel(&self, id: &str) -> Result<(), String> {
        let dir = self.novel_dir(id);
        if dir.exists() {
            std::fs::remove_dir_all(&dir).map_err(|e| format!("删除小说目录失败: {}", e))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Novel;

    #[test]
    fn store_encrypted_files() {
        // 使用临时数据目录（通过环境变量覆盖）
        let dir = std::env::temp_dir().join("jinshu_store_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::env::set_var("JINSHU_DATA_DIR", &dir);
        let store = Store::init().unwrap();
        assert!(store.data_dir.join(".jinshu_key").exists());

        let mut n = Novel::new("加密落盘测试", "作者", "简介", "都市");
        n.add_volume("正文");
        let vid = n.volumes[0].id.clone();
        let cid = n.add_chapter(Some(&vid), "第一章 秘密");
        store.save_novel(&n).unwrap();
        store.save_chapter(&n.meta.id, &cid, "这里是非常机密的章节正文，绝不能明文暴露。").unwrap();

        // 磁盘上的文件必须是密文：明文不应出现在文件中
        let raw = std::fs::read(store.novel_dir(&n.meta.id).join("novel.jsr")).unwrap();
        let raw_str = String::from_utf8_lossy(&raw);
        assert!(!raw_str.contains("加密落盘测试"));
        assert!(!raw_str.contains("第一章 秘密"));
        let ch_raw = std::fs::read(store.novel_dir(&n.meta.id).join("chapters").join(format!("{}.jsr", cid))).unwrap();
        let ch_str = String::from_utf8_lossy(&ch_raw);
        assert!(!ch_str.contains("非常机密"));
        // 无任何明文文件
        let mut has_plain = false;
        fn walk(d: &std::path::Path, has_plain: &mut bool) {
            if let Ok(rd) = std::fs::read_dir(d) {
                for e in rd.flatten() {
                    let p = e.path();
                    if p.is_dir() { walk(&p, has_plain); }
                    else if let Some(ext) = p.extension() {
                        if ext != "jsr" && p.file_name().unwrap_or_default() != ".jinshu_key" {
                            *has_plain = true;
                        }
                    }
                }
            }
        }
        walk(&store.data_dir, &mut has_plain);
        assert!(!has_plain, "数据目录中不应存在非加密文件");

        // 读回验证
        let n2 = store.load_novel(&n.meta.id).unwrap();
        assert_eq!(n2.meta.title, "加密落盘测试");
        let t = store.load_chapter(&n.meta.id, &cid).unwrap();
        assert_eq!(t, "这里是非常机密的章节正文，绝不能明文暴露。");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
