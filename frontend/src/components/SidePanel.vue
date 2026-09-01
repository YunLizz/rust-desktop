<template>
  <aside class="sidepanel">
    <div class="side-head">
      <span class="side-title">{{ headTitle }}</span>
      <button v-if="store.activity === 'chapters'" class="icon-btn" title="新建章节" @click="store.dialog = { kind: 'newChapter' }">＋</button>
      <button v-else-if="store.activity === 'outline'" class="icon-btn" title="添加卷" @click="store.dialog = { kind: 'newVolume' }">＋</button>
      <button v-else-if="store.activity === 'characters'" class="icon-btn" title="新建人物" @click="newCharacter">＋</button>
      <button v-else-if="store.activity === 'world'" class="icon-btn" title="新建地点" @click="newLocation">＋</button>
      <button v-else-if="store.activity === 'timeline'" class="icon-btn" title="新建事件" @click="newEvent">＋</button>
      <button v-else-if="store.activity === 'tasks'" class="icon-btn" title="新建任务链" @click="newChain">＋</button>
    </div>
    <div class="sep" style="margin: 0 10px"></div>

    <!-- 章节树 -->
    <div v-if="store.activity === 'chapters'" class="scroll">
      <div v-for="vol in store.novel?.volumes" :key="vol.id">
        <div class="collapse" :class="{ open: openVols[vol.id] }" @click="toggleVol(vol.id)">
          <span class="arrow">▶</span>
          <span>📁 {{ vol.title }}</span>
          <span class="meta">{{ vol.chapters.length }}章 · {{ volWords(vol) }}字</span>
        </div>
        <div v-show="openVols[vol.id]" style="padding-left: 14px">
          <button
            v-for="c in vol.chapters"
            :key="c.id"
            class="tree-row"
            :class="{ active: c.id === store.activeTab }"
            @click="openCh(c)"
            @contextmenu.prevent="openMenu($event, chapterMenu(c, vol))"
          >
            <span>📄</span>
            <span class="t">{{ c.title }}</span>
            <span class="warn-dot" v-if="store.dirty[c.id]">●</span>
            <span class="meta">{{ c.words }}字</span>
          </button>
          <div class="vol-ops" @contextmenu.prevent="openMenu($event, volumeMenu(vol))">⋯</div>
        </div>
      </div>
      <div v-if="!store.novel?.volumes?.length" class="dim-note">（暂无章节，点上方 ＋ 新建）</div>
    </div>

    <!-- 大纲 -->
    <div v-else-if="store.activity === 'outline'" class="scroll">
      <div class="outline-tools">
        <button class="btn sm" @click="aiOutline">✨ AI 生成</button>
        <button class="btn sm" title="把当前卷/章结构导入大纲树" @click="outlineFromChapters">📋 从章节生成骨架</button>
      </div>
      <OutlineTree :nodes="store.novel?.outline || []" :depth="0" />
    </div>

    <!-- 人物 -->
    <div v-else-if="store.activity === 'characters'" class="scroll">
      <button
        v-for="c in store.novel?.characters"
        :key="c.id"
        class="tree-row"
        :class="{ active: c.id === store.selChar }"
        @click="store.selChar = c.id"
        @contextmenu.prevent="openMenu($event, charMenu(c))"
      >
        <span>👤</span>
        <span class="t">{{ c.name }}</span>
        <span class="tag" :style="roleColor(c.role)">{{ c.role || "其他" }}</span>
      </button>
      <div v-if="!store.novel?.characters?.length" class="dim-note">（暂无人物）</div>
    </div>

    <!-- 世界观 -->
    <div v-else-if="store.activity === 'world'" class="scroll">
      <WorldTree :nodes="rootLocs" :depth="0" />
      <div v-if="!store.novel?.locations?.length" class="dim-note">（暂无地点设定）</div>
    </div>

    <!-- 时间线 -->
    <div v-else-if="store.activity === 'timeline'" class="scroll">
      <button
        v-for="e in sortedEvents"
        :key="e.id"
        class="tree-row"
        :class="{ active: e.id === store.selEvent }"
        @click="store.selEvent = e.id"
        @contextmenu.prevent="openMenu($event, eventMenu(e))"
      >
        <span>⏱️</span>
        <span class="t">{{ e.title }}</span>
        <span class="meta">{{ e.time }}</span>
      </button>
      <div v-if="!store.novel?.timeline?.length" class="dim-note">（暂无事件）</div>
    </div>

    <!-- 任务链 -->
    <div v-else-if="store.activity === 'tasks'" class="scroll">
      <button
        class="tree-row"
        :class="{ active: store.selChain === 'all' }"
        @click="store.selChain = 'all'"
      >
        📋 全部任务
      </button>
      <button
        v-for="ch in store.novel?.chains"
        :key="ch.id"
        class="tree-row"
        :class="{ active: ch.id === store.selChain }"
        @click="store.selChain = ch.id"
        @contextmenu.prevent="openMenu($event, chainMenu(ch))"
      >
        <span>🔗</span>
        <span class="t">{{ ch.name }}</span>
      </button>
      <div v-if="!store.novel?.chains?.length" class="dim-note">（暂无任务链）</div>
    </div>

    <!-- 搜索 -->
    <div v-else-if="store.activity === 'search'" class="scroll">
      <input class="input" style="margin: 8px 10px; width: calc(100% - 20px)" v-model="searchQ" placeholder="输入关键词搜索全部章节…" autofocus />
      <button
        v-for="(r, i) in searchResults"
        :key="i"
        class="search-item"
        @click="gotoResult(r)"
      >
        <div class="si-title">{{ r.title }}</div>
        <div class="si-line">{{ r.line }}</div>
      </button>
      <div v-if="searchQ && !searchResults.length" class="dim-note">无匹配结果</div>
    </div>
  </aside>
</template>

<script setup>
import { ref, computed, h } from "vue";
import { store, toast, openTab, saveAll, allChapters } from "../store";
import { api } from "../api";
import * as prompts from "../prompts";

const headTitle = computed(() => ({
  chapters: "章节",
  outline: "大纲",
  characters: "人物",
  world: "世界观",
  timeline: "时间线",
  tasks: "任务",
  search: "搜索",
}[store.activity] || ""));

const openVols = ref({});
function toggleVol(id) {
  openVols.value[id] = !openVols.value[id];
}
function volWords(vol) {
  return vol.chapters.reduce((a, c) => a + (c.words || 0), 0);
}

function openCh(c) {
  openTab(c.id);
}

// 右键菜单（全局事件）
function openMenu(e, items) {
  window.dispatchEvent(new CustomEvent("jinshu:contextmenu", { detail: { x: e.clientX, y: e.clientY, items } }));
}

function chapterMenu(c, vol) {
  return [
    { label: "打开", run: () => openTab(c.id) },
    { label: "重命名", run: () => (store.dialog = { kind: "renameChapter", payload: { cid: c.id } }) },
    { label: "上移", run: () => moveChapter(c.id, -1) },
    { label: "下移", run: () => moveChapter(c.id, 1) },
    { label: "本卷下新建章节", run: () => (store.dialog = { kind: "newChapter", payload: { volId: vol.id } }) },
    { label: "删除", danger: true, run: () => (store.dialog = { kind: "deleteChapter", payload: { cid: c.id, title: c.title } }) },
  ];
}
function volumeMenu(vol) {
  return [
    { label: "重命名卷", run: () => (store.dialog = { kind: "renameVolume", payload: { vid: vol.id } }) },
    { label: "本卷下新建章节", run: () => (store.dialog = { kind: "newChapter", payload: { volId: vol.id } }) },
    { label: "删除卷（含章节）", danger: true, run: () => (store.dialog = { kind: "deleteVolume", payload: { vid: vol.id, title: vol.title } }) },
  ];
}
function charMenu(c) {
  return [
    { label: "编辑", run: () => (store.selChar = c.id) },
    { label: "重命名", run: () => { const n = prompt("新名字", c.name); if (n) { c.name = n; api.saveNovel(store.novel); } } },
    { label: "删除", danger: true, run: () => {
        store.novel.characters = store.novel.characters.filter((x) => x.id !== c.id);
        if (store.selChar === c.id) store.selChar = null;
        api.saveNovel(store.novel);
      } },
  ];
}
function eventMenu(e) {
  return [
    { label: "编辑", run: () => (store.selEvent = e.id) },
    { label: "删除", danger: true, run: () => {
        store.novel.timeline = store.novel.timeline.filter((x) => x.id !== e.id);
        if (store.selEvent === e.id) store.selEvent = null;
        api.saveNovel(store.novel);
      } },
  ];
}
function chainMenu(ch) {
  return [
    { label: "重命名", run: () => { const n = prompt("新名字", ch.name); if (n) { ch.name = n; api.saveNovel(store.novel); } } },
    { label: "删除任务链", danger: true, run: () => {
        store.novel.tasks.forEach((t) => { if (t.chain_id === ch.id) t.chain_id = null; });
        store.novel.chains = store.novel.chains.filter((x) => x.id !== ch.id);
        api.saveNovel(store.novel);
      } },
  ];
}

function moveChapter(cid, delta) {
  for (const v of store.novel.volumes) {
    const i = v.chapters.findIndex((c) => c.id === cid);
    if (i >= 0) {
      const j = i + delta;
      if (j >= 0 && j < v.chapters.length) {
        const [c] = v.chapters.splice(i, 1);
        v.chapters.splice(j, 0, c);
        api.saveNovel(store.novel);
      }
      return;
    }
  }
}

// 实体新建
function newCharacter() {
  const c = { id: "c" + Math.random().toString(36).slice(2, 8), name: "新人物", role: "", appearance: "", personality: "", background: "", goals: "", notes: "", relationships: [] };
  store.novel.characters.push(c);
  store.selChar = c.id;
  api.saveNovel(store.novel);
}
function newLocation() {
  const l = { id: "l" + Math.random().toString(36).slice(2, 8), name: "新地点", kind: "城市", parent_id: null, description: "" };
  store.novel.locations.push(l);
  store.selLoc = l.id;
  api.saveNovel(store.novel);
}
function newEvent() {
  const e = { id: "e" + Math.random().toString(36).slice(2, 8), title: "新事件", time: "第1卷", description: "", character_ids: [], location_id: null, chapter_id: null };
  store.novel.timeline.push(e);
  store.selEvent = e.id;
  api.saveNovel(store.novel);
}
function newChain() {
  const ch = { id: "h" + Math.random().toString(36).slice(2, 8), name: "新任务链", description: "", task_ids: [] };
  store.novel.chains.push(ch);
  store.selChain = ch.id;
  api.saveNovel(store.novel);
}

// 从章节结构生成大纲骨架
function outlineFromChapters() {
  const n = store.novel;
  if (!n || !n.volumes?.length) {
    toast("暂无章节，先生成章节再生成骨架", false);
    return;
  }
  n.outline = n.volumes.map((v) => ({
    id: "o" + Math.random().toString(36).slice(2, 8),
    title: v.title,
    kind: "卷",
    content: "",
    children: (v.chapters || []).map((c) => ({
      id: "o" + Math.random().toString(36).slice(2, 8),
      title: c.title,
      kind: "章",
      content: "",
      children: [],
    })),
  }));
  api.saveNovel(n);
  toast("已从章节生成大纲骨架（可在详情中补充要点）");
}

// AI 大纲
function aiOutline() {
  import("../store").then((m) => {
    store.aiPanelOpen = true;
    m.startAi("大纲生成", prompts.buildOutline());
  });
}

// 角色标签色
function roleColor(role) {
  const map = { 主角: "var(--accent)", 反派: "var(--danger)", 重要配角: "var(--warn)" };
  const c = map[role] || "var(--text-3)";
  return { color: c, border: `1px solid ${c}33`, background: `${c}1a` };
}

// 世界观树
const rootLocs = computed(() => (store.novel?.locations || []).filter((l) => !l.parent_id));
const locIcon = (k) => ({ 国家: "🏳️", 城市: "🏙️", 地区: "🏞️", 建筑: "🏛️", 异界: "🌌" }[k] || "📍");

function WorldTree(props) {
  const locs = store.novel?.locations || [];
  const children = (p) => locs.filter((l) => l.parent_id === p.id);
  return h("div", {}, [
    ...props.nodes.map((loc) => [
      h("button", {
        class: ["tree-row", { active: loc.id === store.selLoc }],
        onClick: () => (store.selLoc = loc.id),
        onContextmenu: (e) => openMenu(e, [
          { label: "编辑", run: () => (store.selLoc = loc.id) },
          { label: "删除", danger: true, run: () => {
              store.novel.locations = store.novel.locations.filter((x) => x.id !== loc.id && x.parent_id !== loc.id);
              if (store.selLoc === loc.id) store.selLoc = null;
              api.saveNovel(store.novel);
            } },
        ]),
      }, [h("span", {}, locIcon(loc.kind)), h("span", { class: "t" }, loc.name)]),
      children(loc).length ? h(WorldTree, { nodes: children(loc), depth: props.depth + 1 }) : null,
    ]),
  ]);
}

// 大纲树（递归渲染）
function OutlineTree(props) {
  const nodes = props.nodes || [];
  const renderNode = (node, depth) => {
    const kindIcon = node.kind === "卷" ? "📘" : node.kind === "章" ? "📄" : "·";
    const items = [
      { label: "选中", run: () => (store.selOutline = node.id) },
      { label: "添加子卷", run: () => addOutlineChild(node.id, "卷") },
      { label: "添加子章", run: () => addOutlineChild(node.id, "章") },
      { label: "添加节", run: () => addOutlineChild(node.id, "节") },
      { label: "添加要点", run: () => addOutlineChild(node.id, "要点") },
      { label: "上移", run: () => moveOutline(node.id, -1) },
      { label: "下移", run: () => moveOutline(node.id, 1) },
      { label: "删除", danger: true, run: () => deleteOutline(node.id) },
    ];
    return [
      h("button", {
        class: ["tree-row", { active: node.id === store.selOutline }],
        style: { paddingLeft: 6 + depth * 14 + "px" },
        onClick: () => (store.selOutline = node.id),
        onContextmenu: (e) => openMenu(e, items),
      }, [
        h("span", {}, kindIcon),
        h("span", { class: "t" }, node.title),
      ]),
      node.children && node.children.length
        ? h("div", {}, node.children.map((c) => renderNode(c, depth + 1)))
        : null,
    ];
  };
  return h("div", {}, nodes.map((n) => renderNode(n, props.depth || 0)));
}

function addOutlineChild(parentId, kind) {
  const node = { id: "o" + Math.random().toString(36).slice(2, 8), title: "新节点", kind, content: "", children: [] };
  const find = (arr) => {
    for (const n of arr) {
      if (n.id === parentId) { n.children.push(node); return true; }
      if (n.children && find(n.children)) return true;
    }
    return false;
  };
  if (!parentId) store.novel.outline.push(node);
  else find(store.novel.outline);
  store.selOutline = node.id;
  api.saveNovel(store.novel);
}
function moveOutline(id, delta) {
  const rec = (arr) => {
    const i = arr.findIndex((n) => n.id === id);
    if (i >= 0) {
      const j = i + delta;
      if (j >= 0 && j < arr.length) {
        const [n] = arr.splice(i, 1);
        arr.splice(j, 0, n);
        return true;
      }
    }
    for (const n of arr) if (n.children && rec(n.children)) return true;
    return false;
  };
  if (rec(store.novel.outline)) api.saveNovel(store.novel);
}
function deleteOutline(id) {
  const rec = (arr) => {
    const i = arr.findIndex((n) => n.id === id);
    if (i >= 0) { arr.splice(i, 1); return true; }
    for (const n of arr) if (n.children && rec(n.children)) return true;
    return false;
  };
  if (rec(store.novel.outline)) {
    if (store.selOutline === id) store.selOutline = null;
    api.saveNovel(store.novel);
  }
}

// 时间线排序
const sortedEvents = computed(() => [...(store.novel?.timeline || [])].sort((a, b) => a.time.localeCompare(b.time)));

// 搜索
const searchQ = ref("");
const searchResults = computed(() => {
  const q = searchQ.value.trim();
  if (!q || !store.novel) return [];
  const out = [];
  for (const c of allChapters(store.novel)) {
    const text = store.chapters[c.id] || "";
    const line = text.split("\n").find((l) => l.includes(q));
    if (line) out.push({ cid: c.id, title: c.title, line: line.trim().slice(0, 60) });
  }
  return out;
});
function gotoResult(r) {
  openTab(r.cid);
}
</script>

<style scoped>
.sidepanel {
  width: 300px;
  min-width: 220px;
  max-width: 560px;
  flex-shrink: 0;
  background: var(--panel);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  min-height: 0;
  resize: horizontal;
  overflow: hidden;
}
.side-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 10px 6px 14px;
}
.side-title { font-size: 13px; font-weight: 600; }
.scroll { flex: 1; overflow-y: auto; padding: 4px 6px 12px; }
.t { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.meta { margin-left: auto; font-size: 10.5px; color: var(--text-3); flex-shrink: 0; }
.warn-dot { color: var(--warn); font-size: 8px; }
.dim-note { color: var(--text-3); font-size: 11.5px; padding: 10px 12px; }
.vol-ops { color: var(--text-3); font-size: 11px; padding: 2px 10px; cursor: context-menu; }
.outline-tools { display: flex; gap: 5px; padding: 6px 10px; flex-wrap: wrap; }
.vol-ops:hover { color: var(--text-2); }
.search-item { display: block; width: 100%; text-align: left; border: none; background: transparent; padding: 7px 10px; border-radius: 6px; cursor: pointer; font-family: inherit; }
.search-item:hover { background: var(--hover); }
.si-title { font-size: 12.5px; color: var(--text); }
.si-line { font-size: 11px; color: var(--text-2); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
</style>
