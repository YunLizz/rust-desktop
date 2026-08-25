use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct NovelMeta {
    pub id: String,
    pub title: String,
    pub author: String,
    pub description: String,
    pub genre: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub total_words: u64,
    pub chapter_count: u32,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ChapterMeta {
    pub id: String,
    pub title: String,
    pub words: u64,
    pub updated_at: i64,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Volume {
    pub id: String,
    pub title: String,
    pub chapters: Vec<ChapterMeta>,
}

/// 树形大纲节点：卷 / 章 / 节 / 要点
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct OutlineNode {
    pub id: String,
    pub title: String,
    pub kind: String, // 卷 | 章 | 节 | 要点
    pub content: String,
    pub children: Vec<OutlineNode>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Relationship {
    pub target_id: String,
    pub target_name: String,
    pub relation: String, // 如「兄妹」「宿敌」
    pub note: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Character {
    pub id: String,
    pub name: String,
    pub role: String, // 主角 / 重要配角 / 配角 / 反派 / 其他
    pub appearance: String,
    pub personality: String,
    pub background: String,
    pub goals: String,
    pub notes: String,
    pub relationships: Vec<Relationship>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Location {
    pub id: String,
    pub name: String,
    pub kind: String, // 国家 / 城市 / 地区 / 建筑 / 异界 / 其他
    pub parent_id: Option<String>,
    pub description: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct TimelineEvent {
    pub id: String,
    pub title: String,
    pub time: String, // 时间描述：可自由填写，如「第三卷 第12章 前夜」
    pub description: String,
    pub character_ids: Vec<String>,
    pub location_id: Option<String>,
    pub chapter_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: u8, // 0 待办 1 进行中 2 已完成
    pub chain_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct TaskChain {
    pub id: String,
    pub name: String,
    pub description: String,
    pub task_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Novel {
    pub meta: NovelMeta,
    pub volumes: Vec<Volume>,
    pub outline: Vec<OutlineNode>,
    pub characters: Vec<Character>,
    pub locations: Vec<Location>,
    pub timeline: Vec<TimelineEvent>,
    pub tasks: Vec<Task>,
    pub chains: Vec<TaskChain>,
    /// 写作统计：日期 -> 当日新增字数
    pub stats: BTreeMap<String, u64>,
    pub ai_summary: String, // AI 生成的整稿摘要（用于上下文）
}

impl Novel {
    pub fn new(title: &str, author: &str, description: &str, genre: &str) -> Self {
        let ts = crate::util::now_ts();
        let mut meta = NovelMeta {
            id: crate::util::new_id(),
            title: title.trim().to_string(),
            author: author.trim().to_string(),
            description: description.trim().to_string(),
            genre: genre.trim().to_string(),
            created_at: ts,
            updated_at: ts,
            ..Default::default()
        };
        if meta.title.is_empty() {
            meta.title = "未命名小说".into();
        }
        Self {
            meta,
            ..Default::default()
        }
    }

    pub fn chapters_all(&self) -> Vec<&ChapterMeta> {
        self.volumes.iter().flat_map(|v| v.chapters.iter()).collect()
    }

    pub fn find_chapter(&self, cid: &str) -> Option<&ChapterMeta> {
        self.volumes.iter().flat_map(|v| v.chapters.iter()).find(|c| c.id == cid)
    }

    pub fn find_chapter_mut(&mut self, cid: &str) -> Option<&mut ChapterMeta> {
        self.volumes.iter_mut().flat_map(|v| v.chapters.iter_mut()).find(|c| c.id == cid)
    }

    pub fn volume_of_chapter(&self, cid: &str) -> Option<&Volume> {
        self.volumes.iter().find(|v| v.chapters.iter().any(|c| c.id == cid))
    }

    pub fn find_character(&self, id: &str) -> Option<&Character> {
        self.characters.iter().find(|c| c.id == id)
    }

    pub fn find_character_mut(&mut self, id: &str) -> Option<&mut Character> {
        self.characters.iter_mut().find(|c| c.id == id)
    }

    pub fn find_location(&self, id: &str) -> Option<&Location> {
        self.locations.iter().find(|l| l.id == id)
    }

    pub fn find_location_mut(&mut self, id: &str) -> Option<&mut Location> {
        self.locations.iter_mut().find(|l| l.id == id)
    }

    pub fn add_volume(&mut self, title: &str) {
        self.volumes.push(Volume {
            id: crate::util::new_id(),
            title: title.trim().to_string(),
            chapters: Vec::new(),
        });
    }

    pub fn add_chapter(&mut self, vid: Option<&str>, title: &str) -> String {
        let cid = crate::util::new_id();
        let meta = ChapterMeta {
            id: cid.clone(),
            title: title.trim().to_string(),
            words: 0,
            updated_at: crate::util::now_ts(),
        };
        match vid.and_then(|vid| self.volumes.iter_mut().find(|vol| vol.id == vid)) {
            Some(v) => v.chapters.push(meta),
            None => {
                if self.volumes.is_empty() {
                    self.volumes.push(Volume {
                        id: crate::util::new_id(),
                        title: "正文".into(),
                        chapters: Vec::new(),
                    });
                }
                // 上面 is_empty 分支保证至少有 1 个卷，直接放入最后一个卷中
                if let Some(last) = self.volumes.last_mut() {
                    last.chapters.push(meta);
                }
            }
        }
        self.meta.chapter_count += 1;
        cid
    }

    pub fn delete_chapter(&mut self, cid: &str) {
        for v in &mut self.volumes {
            v.chapters.retain(|c| c.id != cid);
        }
        self.volumes.retain(|v| !v.chapters.is_empty() || v.title != "正文" || v.title.is_empty());
        self.meta.chapter_count = self.chapters_all().len() as u32;
        self.timeline.retain(|e| e.chapter_id.as_deref() != Some(cid));
    }

    pub fn total_words(&self) -> u64 {
        self.chapters_all().iter().map(|c| c.words).sum()
    }

    /// 把章节点按「卷-章」结构挂到卷树
    pub fn sync_from_chapters(&mut self) {
        // 仅当卷为空时初始化一个默认卷
        if self.volumes.is_empty() && !self.chapters_all().is_empty() {
            let mut v = Volume {
                id: crate::util::new_id(),
                title: "正文".into(),
                chapters: Vec::new(),
            };
            v.chapters = self.chapters_all().into_iter().cloned().collect();
            self.volumes.push(v);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_count_cjk() {
        // 汉字+标点按字符计，英文按词计
        assert_eq!(crate::util::count_words("你好世界"), 4);
        assert_eq!(crate::util::count_words("你好，世界！"), 6);
        assert_eq!(crate::util::count_words("hello world"), 2);
        assert_eq!(crate::util::count_words("第一章 少年出山"), 7);
        assert_eq!(crate::util::count_words(""), 0);
    }

    #[test]
    fn novel_chapter_flow() {
        let mut n = Novel::new("测试之书", "作者", "简介", "玄幻");
        n.add_volume("第一卷");
        let vid = n.volumes[0].id.clone();
        let cid = n.add_chapter(Some(&vid), "第一章 测试");
        assert_eq!(n.chapters_all().len(), 1);
        let c = n.find_chapter(&cid).unwrap();
        assert_eq!(c.title, "第一章 测试");
        n.delete_chapter(&cid);
        assert_eq!(n.chapters_all().len(), 0);
    }
}
