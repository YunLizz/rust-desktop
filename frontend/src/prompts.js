// AI 提示词构建（JS 版，与后端 Rust 版同一套设计）
import { store, today } from "./store";

const truncate = (s, n) => (s.length > n ? s.slice(0, n) + "…" : s);

export function outlineText() {
  const n = store.novel;
  if (!n || !n.outline?.length) return "（暂无大纲）";
  const walk = (nodes, depth) =>
    nodes
      .map((nd) => "  ".repeat(depth) + nd.title + (nd.content ? "：" + nd.content : "") + "\n" + walk(nd.children || [], depth + 1))
      .join("");
  return walk(n.outline, 0);
}

export function charactersText(names) {
  const n = store.novel;
  if (!n) return "";
  let out = "";
  const list = names?.length ? names : n.characters.map((c) => c.name);
  for (const name of list) {
    const c = n.characters.find((x) => x.name === name || x.name.includes(name));
    if (c) out += `【${c.name}】（${c.role || "其他"}）：外貌=${c.appearance}；性格=${c.personality}；背景=${c.background}；目标=${c.goals}\n`;
  }
  if (!out && n.characters.length) out += "人物列表：" + n.characters.map((c) => c.name).join("、");
  return out;
}

export function worldText() {
  const n = store.novel;
  if (!n) return "";
  const out = n.locations
    .filter((l) => l.description)
    .map((l) => `【${l.name}】（${l.kind || "地点"}）${l.description}`)
    .join("\n");
  return out || "（暂无世界观设定）";
}

export function loreHits(text) {
  const n = store.novel;
  if (!n) return { chars: [], locs: [] };
  const chars = n.characters.filter((c) => c.name.length >= 2 && text.includes(c.name)).map((c) => c.name);
  const locs = n.locations.filter((l) => l.name.length >= 2 && text.includes(l.name)).map((l) => l.name);
  return { chars, locs };
}

export function buildContext(chapterText, injectLore) {
  const outline = outlineText();
  const world = worldText();
  let chars = "";
  if (injectLore) {
    const tail = chapterText.slice(-3000);
    const { chars: ch, locs } = loreHits(tail);
    chars = charactersText([...ch, ...locs]);
  } else {
    chars = charactersText(null);
  }
  return { outline, chars, world };
}

const chapterTail = (s, n) => (s.length > n ? "……（前文略）\n" + s.slice(-n) : s);

// ---------- 续写 ----------
export function buildContinue(instruction = "") {
  const n = store.novel;
  const cid = store.activeTab;
  const title = chapterTitleOf(cid);
  const text = store.chapters[cid] || "";
  const { outline, chars, world } = buildContext(text, store.useLore);
  return [
    `你正在续写小说《${n.meta.title}》的章节《${title}》。

作品大纲：
${outline}

已出场人物：
${chars || "（无）"}

世界观：
${world}

当前章节已有的正文（末尾部分）：
=====
${chapterTail(text, 2500)}
=====

请紧接上文续写正文，要求：
1. 文风、人称、视角与上文保持一致，不跳脱
2. 符合大纲走向，不引入大纲外的新势力/新设定（如需引入需自然铺垫）
3. 对话与动作描写比例均衡，注重画面感
4. 直接输出续写正文，不要解释、不要加标题、不要用引号包裹
5. 字数 600~1200 字，在情节推进处自然收尾

补充要求：${instruction || "无"}`,
  ];
}

function chapterTitleOf(cid) {
  return allChapters(store.novel).find((c) => c.id === cid)?.title || "";
}

// ---------- 大纲生成 ----------
export function buildOutline() {
  const n = store.novel;
  return [
    `请为小说《${n.meta.title}》生成完整的创作大纲。

题材：${n.meta.genre || "未指定"}
核心设定/一句话简介：${n.meta.description || "（未填写）"}
目标篇幅：长篇（约 100 万字）

请按以下格式输出（严格遵循，不要添加其他内容）：
【卷一：卷标题】
- 第1章 章节标题：本章剧情要点（1~2句）
- 第2章 章节标题：本章剧情要点
…
【卷二：卷标题】
- …

要求：
1. 每卷 5~10 章，按目标篇幅控制总章数
2. 情节有起承转合，卷末留有钩子
3. 标注伏笔与人物成长线索（可在要点后加【伏笔：…】）`,
  ];
}

export function parseOutline(text) {
  const volumes = [];
  let cur = -1;
  for (const raw of text.split("\n")) {
    const line = raw.trim();
    if (!line) continue;
    if (line.startsWith("【")) {
      const title = line.replace(/^【/, "").split("】")[0].trim();
      if (title) {
        volumes.push({ id: "o" + Math.random().toString(36).slice(2, 8), title, kind: "卷", content: "", children: [] });
        cur = volumes.length - 1;
      }
    } else if (line.startsWith("-") || line.startsWith("*")) {
      const body = line.replace(/^[-*\s]+/, "").trim();
      const idx = body.indexOf("：") >= 0 ? body.indexOf("：") : body.indexOf(":");
      const title = idx >= 0 ? body.slice(0, idx).trim() : body;
      const content = idx >= 0 ? body.slice(idx + 1).trim() : "";
      if (!title) continue;
      const node = { id: "o" + Math.random().toString(36).slice(2, 8), title, kind: "章", content, children: [] };
      if (cur >= 0) volumes[cur].children.push(node);
      else {
        volumes.push({ id: "o" + Math.random().toString(36).slice(2, 8), title: "第一卷", kind: "卷", content: "", children: [node] });
        cur = volumes.length - 1;
      }
    } else if (cur >= 0 && volumes[cur].children.length) {
      volumes[cur].children[volumes[cur].children.length - 1].content += " " + line;
    }
  }
  return volumes;
}

// ---------- 细纲 / 润色 / 扩写 / 摘要 ----------
export function buildChapterOutline() {
  const n = store.novel;
  const cid = store.activeTab;
  const title = chapterTitleOf(cid);
  const text = store.chapters[cid] || "";
  const { outline, chars } = buildContext(text, false);
  return [
    `为小说《${n.meta.title}》的章节《${title}》生成写作细纲。

作品大纲：
${outline}

人物：
${chars || "（无）"}

章节已有内容：
${chapterTail(text, 2000)}

请输出：
1. 本回目目标（推进什么、揭示什么、埋什么伏笔）
2. 分场次结构：场景1（地点/出场人物/发生的事/情绪节奏）
3. 结尾钩子建议
4. 3 个可选的意外/转折方向

简明扼要，不要空话。`,
  ];
}

export function buildPolish(sel, instruction) {
  return [
    `请对下面的文本进行润色改写。
要求：${instruction || "保持原意，提升文笔、画面感与节奏"}
直接输出润色后的完整文本，不要解释，不要加引号。

=====
${sel}
=====`,
  ];
}

export function buildExpand(sel, instruction) {
  return [
    `请扩写下面的文本：补充细节、氛围、动作与心理描写，让节奏更饱满。
要求：${instruction || "扩写到原来的 2~3 倍，保持文风一致"}
直接输出扩写后的完整文本，不要解释。

=====
${sel}
=====`,
  ];
}

export function buildSummary() {
  const cid = store.activeTab;
  const title = chapterTitleOf(cid);
  const text = (store.chapters[cid] || "").slice(0, 6000);
  return [
    `请用 150 字以内概括章节《${title}》的情节，要求：包含出场人物、关键事件、留下的伏笔。
只输出摘要正文，不要其他内容。

=====
${text}
=====`,
  ];
}

// ---------- 逻辑 / 剧情 / 一致性 / 评审 ----------
export function buildLogicCheck() {
  const n = store.novel;
  const cid = store.activeTab;
  const title = chapterTitleOf(cid);
  const text = (store.chapters[cid] || "").slice(0, 4000);
  const { outline, world } = buildContext(text, false);
  return [
    `请检查小说《${n.meta.title}》当前章节《${title}》的逻辑问题。

大纲：
${outline}

世界观/设定：
${world}

章节正文：
${text}

请输出：
1. 情节漏洞与逻辑硬伤（引用原文佐证）
2. 人物行为是否违背人设
3. 时间线/设定冲突
4. 修改建议

若无问题请直接说"未发现明显逻辑问题"。`,
  ];
}

export function buildPlotIdeas() {
  const n = store.novel;
  const cid = store.activeTab;
  const title = chapterTitleOf(cid);
  const text = (store.chapters[cid] || "").slice(-1500);
  const { outline, chars, world } = buildContext(text, false);
  return [
    `作为小说《${n.meta.title}》的剧情顾问，请给出后续剧情的 5 个发展方向。

大纲：
${outline}

人物：
${chars || "（无）"}

世界观：
${world}

当前章节《${title}》末尾：
${text}

每个方向给出：名称、一句话概述、潜在冲突、风险点。
要新颖、符合题材调性、能推动主线。`,
  ];
}

export function chapterSummaries() {
  const n = store.novel;
  return allChapters(n).map((c) => {
    const s = store.summaries[c.id];
    if (s) return { title: c.title, text: s };
    const head = (store.chapters[c.id] || "").slice(0, 120);
    return { title: c.title, text: `（无摘要，开头：${head}…）` };
  });
}

export function buildConsistency() {
  const n = store.novel;
  const chars = charactersText(null);
  const chaps = chapterSummaries().map((c) => `《${c.title}》：${c.text}`).join("\n");
  return [
    `请对小说《${n.meta.title}》进行全稿一致性检查。

人物设定：
${chars || "（无）"}

各章节摘要：
${chaps}

请检查：
1. 人物言行/能力/关系前后是否矛盾
2. 时间线是否错乱
3. 已写设定（地名/道具/规则）是否前后一致
4. 伏笔是否被回收或遗忘
按严重程度列出问题清单，附修改建议。`,
  ];
}

export function buildFeedback() {
  const n = store.novel;
  const chaps = chapterSummaries().map((c) => `《${c.title}》：${c.text}`).join("\n");
  return [
    `请对小说《${n.meta.title}》做一次整稿评审（类似资深编辑的反馈）。

简介：${n.meta.description}
大纲：
${outlineText()}

章节摘要：
${chaps}

请从 5 个维度给出结构化反馈，每项用「建议」结尾：
1. 剧情节奏（开篇钩子、中间拖沓点、高潮分布）
2. 人物塑造（弧光是否完整、对话是否千人一面）
3. 结构连贯性（线索埋设与回收、卷章衔接）
4. 文笔风格（描写密度、重复用词、对话质量）
5. 读者体验（爽点/情绪点密度、可能的弃书点）
最后给出 3 条最优先的改进动作。`,
  ];
}

// ---------- 人物卡 / 世界观 / 起名 / 简介 ----------
export function buildCharacterCard(name, role) {
  const n = store.novel;
  return [
    `为小说《${n.meta.title}》设计人物「${name}」（角色定位：${role || "主角"}）。
简介：${n.meta.description}

请输出人物卡：
外貌（3~5句，有辨识度）
性格（核心特质+矛盾点）
背景经历（影响其动机的关键事件）
目标与欲望（表层/深层）
说话风格（举例1句台词）
与主线的潜在关联`,
  ];
}

export function buildWorld() {
  const n = store.novel;
  return [
    `为小说《${n.meta.title}》设计「主世界」（类别：世界观）。
简介：${n.meta.description}
已有设定：
${worldText()}

请输出：
1. 核心规则/设定（3~5条硬规则，明确边界）
2. 社会结构（势力、阶层、组织）
3. 2~3个标志性地点及其氛围
4. 可驱动的剧情冲突点`,
  ];
}

export function buildNaming() {
  const n = store.novel;
  return [
    `请为小说《${n.meta.title}》（题材：${n.meta.genre || "未指定"}）的人物起名，给 8 个候选。
灵感/限制：古典、有仙气
要求：贴合题材气质、朗朗上口、避免常见俗名，每个名字附一句说明。
格式：
1. 名字 —— 说明`,
  ];
}

export function buildSynopsis() {
  const n = store.novel;
  return [
    `为小说《${n.meta.title}》写一版平台宣传简介。
题材：${n.meta.genre || "未指定"}
设定/卖点：${n.meta.description}
大纲：
${outlineText()}

要求：
1. 150 字以内，前 3 句必须抓住人
2. 突出独特卖点与核心冲突
3. 给出 2 个备选（一个悬念向、一个爽点向）`,
  ];
}

export function buildFreeChat(input) {
  const n = store.novel;
  const { outline, chars, world } = buildContext(store.chapters[store.activeTab] || "", store.useLore);
  return [
    `自由提问（当前作品《${n.meta.title}》，已有大纲：${truncate(outline, 800)}；人物：${truncate(chars, 800)}；世界观：${truncate(world, 600)}）：\n${input}`,
  ];
}
