//! 提示词模板：面向中文小说创作场景的结构化指令

use crate::model::Novel;
use crate::util;

fn wrap_user(content: String) -> Vec<(String, String)> {
    vec![("user".into(), content)]
}

/// 大纲的文本化表达（用于 AI 上下文）
pub fn outline_to_text(novel: &Novel) -> String {
    fn walk(nodes: &[crate::model::OutlineNode], depth: usize, out: &mut String) {
        for n in nodes {
            out.push_str(&"  ".repeat(depth));
            out.push_str(&n.title);
            if !n.content.is_empty() {
                out.push_str(&format!("：{}", n.content));
            }
            out.push('\n');
            walk(&n.children, depth + 1, out);
        }
    }
    if novel.outline.is_empty() {
        return "（暂无大纲）".into();
    }
    let mut s = String::new();
    walk(&novel.outline, 0, &mut s);
    s
}

/// 人物设定上下文（Lorebook 式注入）
pub fn characters_to_text(novel: &Novel, names: &[String]) -> String {
    let mut out = String::new();
    for name in names {
        if let Some(c) = novel
            .characters
            .iter()
            .find(|c| c.name == *name || c.name.contains(name.as_str()))
        {
            out.push_str(&format!(
                "【{}】（{}）：外貌={}；性格={}；背景={}；目标={}",
                c.name, c.role, c.appearance, c.personality, c.background, c.goals
            ));
            out.push('\n');
        }
    }
    if out.is_empty() {
        let all: Vec<String> = novel.characters.iter().map(|c| c.name.clone()).collect();
        if !all.is_empty() {
            out.push_str(&format!("人物列表：{}", all.join("、")));
        }
    }
    out
}

/// 世界观上下文
pub fn world_to_text(novel: &Novel) -> String {
    let mut out = String::new();
    for loc in &novel.locations {
        if !loc.description.trim().is_empty() {
            out.push_str(&format!("【{}】（{}）{}", loc.name, loc.kind, loc.description));
            out.push('\n');
        }
    }
    if out.is_empty() {
        "（暂无世界观设定）".into()
    } else {
        out
    }
}

/// 从最近正文中提取出现的人物/地名（关键词触发式注入）
pub fn lore_hits(novel: &Novel, text: &str) -> (Vec<String>, Vec<String>) {
    let mut chars = Vec::new();
    let mut locs = Vec::new();
    for c in &novel.characters {
        if c.name.chars().count() >= 2 && text.contains(&c.name) {
            chars.push(c.name.clone());
        }
    }
    for l in &novel.locations {
        if l.name.chars().count() >= 2 && text.contains(&l.name) {
            locs.push(l.name.clone());
        }
    }
    (chars, locs)
}

pub fn build_context(
    novel: &Novel,
    chapter_text: &str,
    inject_lore: bool,
) -> (String, String, String) {
    let outline = outline_to_text(novel);
    let world = world_to_text(novel);
    let (hits, loc_hits) = if inject_lore {
        lore_hits(novel, &util::tail_chars(chapter_text, 3000))
    } else {
        (Vec::new(), Vec::new())
    };
    let mut names = hits;
    names.extend(loc_hits);
    let chars = characters_to_text(novel, &names);
    (outline, chars, world)
}

// ---------- 续写 ----------
pub fn build_continue(
    novel: &Novel,
    chapter_text: &str,
    chapter_title: &str,
    instruction: &str,
    inject_lore: bool,
) -> Vec<(String, String)> {
    let (outline, chars, world) = build_context(novel, chapter_text, inject_lore);
    let tail = util::tail_chars(chapter_text, 2500);
    let p = format!(
        "你正在续写小说《{}》的章节《{}》。\n\n\
         作品大纲：\n{}\n\n\
         已出场人物：\n{}\n\n\
         世界观：\n{}\n\n\
         当前章节已有的正文（末尾部分）：\n\
         =====\n{}\n=====\n\n\
         请紧接上文续写正文，要求：\n\
         1. 文风、人称、视角与上文保持一致，不跳脱\n\
         2. 符合大纲走向，不引入大纲外的新势力/新设定（如需引入需自然铺垫）\n\
         3. 对话与动作描写比例均衡，注重画面感\n\
         4. 直接输出续写正文，不要解释、不要加标题、不要用引号包裹\n\
         5. 字数 600~1200 字，在情节推进处自然收尾\n\n\
         补充要求：{}",
        novel.meta.title, chapter_title, outline, chars, world, tail, instruction
    );
    wrap_user(p)
}

// ---------- 大纲生成 ----------
pub fn build_outline(
    title: &str,
    premise: &str,
    genre: &str,
    target_len: &str,
    extra: &str,
) -> Vec<(String, String)> {
    let p = format!(
        "请为小说《{}》生成完整的创作大纲。\n\n\
         题材：{}\n\
         核心设定/一句话简介：{}\n\
         目标篇幅：{}\n\
         额外要求：{}\n\n\
         请按以下格式输出（严格遵循，不要添加其他内容）：\n\
         【卷一：卷标题】\n\
         - 第1章 章节标题：本章剧情要点（1~2句）\n\
         - 第2章 章节标题：本章剧情要点\n\
         …\n\
         【卷二：卷标题】\n\
         - …\n\n\
         要求：\n\
         1. 每卷 5~10 章，按目标篇幅控制总章数\n\
         2. 情节有起承转合，卷末留有钩子\n\
         3. 标注伏笔与人物成长线索（可在要点后加【伏笔：…】）",
        title, genre, premise, target_len, extra
    );
    wrap_user(p)
}

/// 解析 AI 大纲输出为树形结构
pub fn parse_outline(text: &str) -> Vec<crate::model::OutlineNode> {
    let mut volumes: Vec<crate::model::OutlineNode> = Vec::new();
    let mut cur_vol: Option<usize> = None;
    let mut pending: Option<crate::model::OutlineNode> = None;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with("【") {
            let title = line.trim_start_matches('【').split('】').next().unwrap_or("").trim().to_string();
            if !title.is_empty() {
                volumes.push(crate::model::OutlineNode {
                    id: util::new_id(),
                    title,
                    kind: "卷".into(),
                    content: String::new(),
                    children: Vec::new(),
                });
                cur_vol = Some(volumes.len() - 1);
                pending = None;
            }
        } else if line.starts_with('-') || line.starts_with('*') {
            let body = line.trim_start_matches(['-', '*', ' ']).trim();
            let (title, content) = match body
                .char_indices()
                .find(|(_, c)| *c == '：' || *c == ':')
            {
                Some((idx, c)) => (
                    body[..idx].trim().to_string(),
                    body[idx + c.len_utf8()..].trim().to_string(),
                ),
                None => (body.to_string(), String::new()),
            };
            if title.is_empty() {
                continue;
            }
            let node = crate::model::OutlineNode {
                id: util::new_id(),
                title,
                kind: "章".into(),
                content,
                children: Vec::new(),
            };
            match cur_vol {
                Some(vi) => volumes[vi].children.push(node),
                None => {
                    if let Some(p) = pending.take() {
                        volumes.push(p);
                    }
                    let idx = volumes.len();
                    volumes.push(crate::model::OutlineNode {
                        id: util::new_id(),
                        title: "第一卷".into(),
                        kind: "卷".into(),
                        content: String::new(),
                        children: vec![node],
                    });
                    cur_vol = Some(idx);
                }
            }
        } else if line.contains('：') || line.contains(':') {
            // 卷标题兜底（无【】包裹时）
            let title = line.split('：').next().unwrap_or(line).trim().to_string();
            if !title.is_empty() && title.chars().count() <= 12 {
                volumes.push(crate::model::OutlineNode {
                    id: util::new_id(),
                    title,
                    kind: "卷".into(),
                    content: String::new(),
                    children: Vec::new(),
                });
                cur_vol = Some(volumes.len() - 1);
            }
        } else {
            // 要点行
            if let Some(vi) = cur_vol {
                if let Some(last) = volumes[vi].children.last_mut() {
                    last.content.push(' ');
                    last.content.push_str(line);
                }
            }
        }
    }
    if let Some(p) = pending {
        volumes.push(p);
    }
    volumes
}

// ---------- 本章细纲 ----------
pub fn build_chapter_outline(
    novel: &Novel,
    chapter_text: &str,
    chapter_title: &str,
) -> Vec<(String, String)> {
    let (outline, chars, _) = build_context(novel, chapter_text, false);
    let p = format!(
        "为小说《{}》的章节《{}》生成写作细纲。\n\n\
         作品大纲：\n{}\n\n\
         人物：\n{}\n\n\
         章节已有内容：\n{}\n\n\
         请输出：\n\
         1. 本回目的（推进什么、揭示什么、埋什么伏笔）\n\
         2. 分场次结构：场景1（地点/出场人物/发生的事/情绪节奏）\n\
         3. 结尾钩子建议\n\
         4. 3 个可选的意外/转折方向\n\n\
         简明扼要，不要空话。",
        novel.meta.title, chapter_title, outline, chars, util::truncate_chars(chapter_text, 2000)
    );
    wrap_user(p)
}

// ---------- 润色 / 扩写 ----------
pub fn build_polish(selected: &str, instruction: &str) -> Vec<(String, String)> {
    let p = format!(
        "请对下面的文本进行润色改写。\n\
         要求：{}\n\
         直接输出润色后的完整文本，不要解释，不要加引号。\n\n\
         =====\n{}\n=====",
        if instruction.trim().is_empty() { "保持原意，提升文笔、画面感与节奏" } else { instruction },
        selected
    );
    wrap_user(p)
}

pub fn build_expand(selected: &str, instruction: &str) -> Vec<(String, String)> {
    let p = format!(
        "请扩写下面的文本：补充细节、氛围、动作与心理描写，让节奏更饱满。\n\
         要求：{}\n\
         直接输出扩写后的完整文本，不要解释。\n\n\
         =====\n{}\n=====",
        if instruction.trim().is_empty() { "扩写到原来的 2~3 倍，保持文风一致" } else { instruction },
        selected
    );
    wrap_user(p)
}

// ---------- 摘要 ----------
pub fn build_summary(chapter_title: &str, text: &str) -> Vec<(String, String)> {
    let p = format!(
        "请用 150 字以内概括章节《{}》的情节，要求：包含出场人物、关键事件、留下的伏笔。\n\
         只输出摘要正文，不要其他内容。\n\n\
         =====\n{}\n=====",
        chapter_title, util::truncate_chars(text, 6000)
    );
    wrap_user(p)
}

// ---------- 逻辑整理 ----------
pub fn build_logic_check(novel: &Novel, chapter_text: &str, chapter_title: &str) -> Vec<(String, String)> {
    let (outline, _, world) = build_context(novel, chapter_text, false);
    let p = format!(
        "请检查小说《{}》当前章节《{}》的逻辑问题。\n\n\
         大纲：\n{}\n\n\
         世界观/设定：\n{}\n\n\
         章节正文：\n{}\n\n\
         请输出：\n\
         1. 情节漏洞与逻辑硬伤（引用原文佐证）\n\
         2. 人物行为是否违背人设\n\
         3. 时间线/设定冲突\n\
         4. 修改建议\n\n\
         若无问题请直接说“未发现明显逻辑问题”。",
        novel.meta.title, chapter_title, outline, world, util::truncate_chars(chapter_text, 4000)
    );
    wrap_user(p)
}

// ---------- 剧情提示 ----------
pub fn build_plot_ideas(
    novel: &Novel,
    chapter_text: &str,
    chapter_title: &str,
) -> Vec<(String, String)> {
    let (outline, chars, world) = build_context(novel, chapter_text, false);
    let p = format!(
        "作为小说《{}》的剧情顾问，请给出后续剧情的 5 个发展方向。\n\n\
         大纲：\n{}\n\n\
         人物：\n{}\n\n\
         世界观：\n{}\n\n\
         当前章节《{}》末尾：\n{}\n\n\
         每个方向给出：名称、一句话概述、潜在冲突、风险点。\n\
         要新颖、符合题材调性、能推动主线。",
        novel.meta.title, outline, chars, world, chapter_title, util::tail_chars(chapter_text, 1500)
    );
    wrap_user(p)
}

// ---------- 一致性检查（整稿） ----------
pub fn build_consistency(novel: &Novel, chapter_summaries: &[(String, String)]) -> Vec<(String, String)> {
    let mut chapters_text = String::new();
    for (t, s) in chapter_summaries {
        chapters_text.push_str(&format!("《{}》：{}\n", t, s));
    }
    let chars = characters_to_text(novel, &novel.characters.iter().map(|c| c.name.clone()).collect::<Vec<_>>());
    let p = format!(
        "请对小说《{}》进行全稿一致性检查。\n\n\
         人物设定：\n{}\n\n\
         各章节摘要：\n{}\n\n\
         请检查：\n\
         1. 人物言行/能力/关系前后是否矛盾\n\
         2. 时间线是否错乱\n\
         3. 已写设定（地名/道具/规则）是否前后一致\n\
         4. 伏笔是否被回收或遗忘\n\
         按严重程度列出问题清单，附修改建议。",
        novel.meta.title, chars, chapters_text
    );
    wrap_user(p)
}

// ---------- 整稿评审 ----------
pub fn build_feedback(novel: &Novel, chapter_summaries: &[(String, String)]) -> Vec<(String, String)> {
    let mut chapters_text = String::new();
    for (t, s) in chapter_summaries {
        chapters_text.push_str(&format!("《{}》：{}\n", t, s));
    }
    let p = format!(
        "请对小说《{}》做一次整稿评审（类似资深编辑的反馈）。\n\n\
         简介：{}\n\
         大纲：\n{}\n\n\
         章节摘要：\n{}\n\n\
         请从 5 个维度给出结构化反馈，每项用「建议」结尾：\n\
         1. 剧情节奏（开篇钩子、中间拖沓点、高潮分布）\n\
         2. 人物塑造（弧光是否完整、对话是否千人一面）\n\
         3. 结构连贯性（线索埋设与回收、卷章衔接）\n\
         4. 文笔风格（描写密度、重复用词、对话质量）\n\
         5. 读者体验（爽点/情绪点密度、可能的弃书点）\n\
         最后给出 3 条最优先的改进动作。",
        novel.meta.title, novel.meta.description, outline_to_text(novel), chapters_text
    );
    wrap_user(p)
}

// ---------- 人物卡 ----------
pub fn build_character_card(
    novel: &Novel,
    name: &str,
    role: &str,
    extra: &str,
) -> Vec<(String, String)> {
    let p = format!(
        "为小说《{}》设计人物「{}」（角色定位：{}）。\n\
         简介：{}\n\n\
         请输出人物卡：\n\
         外貌（3~5句，有辨识度）\n\
         性格（核心特质+矛盾点）\n\
         背景经历（影响其动机的关键事件）\n\
         目标与欲望（表层/深层）\n\
         说话风格（举例1句台词）\n\
         与主线的潜在关联\n\
         额外要求：{}",
        novel.meta.title, name, role, novel.meta.description, extra
    );
    wrap_user(p)
}

// ---------- 世界观 ----------
pub fn build_world(novel: &Novel, name: &str, kind: &str, extra: &str) -> Vec<(String, String)> {
    let p = format!(
        "为小说《{}》设计「{}」（类别：{}）。\n\
         简介：{}\n\
         已有设定：{}\n\n\
         请输出：\n\
         1. 核心规则/设定（3~5条硬规则，明确边界）\n\
         2. 社会结构（势力、阶层、组织）\n\
         3. 2~3个标志性地点及其氛围\n\
         4. 可驱动的剧情冲突点\n\
         额外要求：{}",
        novel.meta.title,
        name,
        kind,
        novel.meta.description,
        world_to_text(novel),
        extra
    );
    wrap_user(p)
}

// ---------- 起名 ----------
pub fn build_naming(novel: &Novel, seed: &str, kind: &str, count: usize) -> Vec<(String, String)> {
    let p = format!(
        "请为小说《{}》（题材：{}）的{}起名，给 {} 个候选。\n\
         灵感/限制：{}\n\
         要求：贴合题材气质、朗朗上口、避免常见俗名，每个名字附一句说明。\n\
         格式：\n1. 名字 —— 说明",
        novel.meta.title, novel.meta.genre, kind, count, seed
    );
    wrap_user(p)
}

// ---------- 简介 ----------
pub fn build_synopsis(novel: &Novel) -> Vec<(String, String)> {
    let p = format!(
        "为小说《{}》写一版平台宣传简介。\n\
         题材：{}\n\
         设定/卖点：{}\n\
         大纲：\n{}\n\n\
         要求：\n\
         1. 150 字以内，前 3 句必须抓住人\n\
         2. 突出独特卖点与核心冲突\n\
         3. 给出 2 个备选（一个悬念向、一个爽点向）",
        novel.meta.title, novel.meta.genre, novel.meta.description, outline_to_text(novel)
    );
    wrap_user(p)
}

/// 根据 AI 消息文本猜测动作类型（用于按钮文案）
pub fn guess_action(instruction: &str) -> String {
    if instruction.contains("续写") {
        "续写".into()
    } else if instruction.contains("大纲") {
        "大纲".into()
    } else if instruction.contains("润色") || instruction.contains("改写") {
        "润色".into()
    } else if instruction.contains("扩写") {
        "扩写".into()
    } else if instruction.contains("摘要") {
        "摘要".into()
    } else if instruction.contains("逻辑") {
        "逻辑检查".into()
    } else if instruction.contains("评审") || instruction.contains("反馈") {
        "整稿评审".into()
    } else if instruction.contains("人物") {
        "人物卡".into()
    } else if instruction.contains("世界") || instruction.contains("地点") {
        "世界观".into()
    } else if instruction.contains("细纲") {
        "细纲".into()
    } else if instruction.contains("简介") {
        "简介".into()
    } else if instruction.contains("起名") {
        "起名".into()
    } else if instruction.contains("提示") || instruction.contains("灵感") {
        "剧情提示".into()
    } else {
        "对话".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outline_parse() {
        let text = "【卷一：风起】
- 第1章 少年出山：主角下山，遭遇袭击
- 第2章 初露锋芒：……
【卷二：云涌】
- 第1章 宗门大比：……";
        let nodes = parse_outline(text);
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].kind, "卷");
        assert_eq!(nodes[0].title, "卷一：风起");
        assert_eq!(nodes[0].children.len(), 2);
        assert_eq!(nodes[0].children[0].title, "第1章 少年出山");
        assert!(nodes[0].children[0].content.contains("主角下山"));
    }
}
