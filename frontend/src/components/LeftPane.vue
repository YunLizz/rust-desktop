<template>
  <aside class="left-pane" :style="{ width: store.leftWidth + 'px' }">
    <!-- 4 个树 Tab -->
    <div class="lp-tabs">
      <button
        v-for="t in treeTabs"
        :key="t.id"
        class="lp-tab"
        :class="{ active: store.leftTree === t.id }"
        :title="t.tip"
        @click="store.leftTree = t.id"
      >
        <span class="lt-icon">{{ t.icon }}</span>
        <span class="lt-name">{{ t.name }}</span>
        <span class="lt-count" v-if="t.count && t.count()">{{ t.count() }}</span>
      </button>
    </div>

    <!-- 顶部操作条 -->
    <div class="lp-ops">
      <input
        v-if="store.leftTree === 'chapters' || store.leftTree === 'characters'"
        class="lp-search"
        v-model="filter"
        :placeholder="store.leftTree === 'chapters' ? '筛选章节…' : '筛选人物…'"
      />
      <span v-else class="lp-label">{{ treeLabel }}</span>
      <button class="icon-btn" :title="addTip" @click="addCurrent">＋</button>
    </div>

    <!-- ===== 1. 章节树 ===== -->
    <div v-if="store.leftTree === 'chapters'" class="lp-scroll">
      <template v-for="vol in visibleVolumes" :key="vol.id">
        <div class="collapse" :class="{ open: volOpen[vol.id] !== false }" @click="toggleVol(vol.id)">
          <span class="arrow">▶</span>
          <span class="vol-name">{{ vol.title }}</span>
          <span class="meta">{{ vol.chapters.length }}章 {{ volWords(vol) }}字</span>
        </div>
        <div v-show="volOpen[vol.id] !== false" class="indent">
          <button
            v-for="c in vol.chapters"
            :key="c.id"
            class="row ch-row"
            :class="{ active: c.id === store.activeTab }"
            @click="openTab(c.id)"
            @contextmenu.prevent="chapterMenu($event, c)"
          >
            <span class="ch-dot" v-if="store.dirty[c.id]">●</span>
            <span class="row-name">{{ c.title }}</span>
            <span class="row-num">{{ c.words || 0 }}</span>
          </button>
        </div>
      </template>
      <div v-if="!visibleVolumes.length" class="lp-note">
        {{ filter ? "无匹配章节" : "还没有章节，点右上 ＋ 新建" }}
      </div>
      <div class="lp-foot" @click="store.dialog = { kind: 'newVolume' }">＋ 新建卷</div>
    </div>

    <!-- ===== 2. 大纲树 ===== -->
    <div v-else-if="store.leftTree === 'outline'" class="lp-scroll">
      <TreeNode
        v-for="n in store.novel?.outline || []"
        :key="n.id"
        :node="n"
        :depth="0"
        kind="outline"
      />
      <div v-if="!(store.novel?.outline || []).length" class="lp-note">
        还没有大纲<br />点右上 ＋ 添加，或用「设定」→「从章节生成」
      </div>
    </div>

    <!-- ===== 3. 人物树 ===== -->
    <div v-else-if="store.leftTree === 'characters'" class="lp-scroll">
      <div v-for="grp in charGroups" :key="grp.role">
        <div class="count-head">{{ grp.role }}（{{ grp.list.length }}）</div>
        <button
          v-for="c in grp.list"
          :key="c.id"
          class="row p-row"
          :class="{ active: c.id === store.selChar }"
          @click="selectChar(c.id)"
          @contextmenu.prevent="charMenu($event, c)"
        >
          <span class="p-avatar">{{ c.name.slice(0, 1) }}</span>
          <span class="row-name">{{ c.name }}</span>
          <span class="p-badge" v-if="appearedIds.has(c.id)" title="本章正文中出现">本</span>
        </button>
      </div>
      <div v-if="!(store.novel?.characters || []).length" class="lp-note">还没有人物，点右上 ＋ 新建</div>
    </div>

    <!-- ===== 5. 任务线（角色的任务，按角色分组） ===== -->
    <div v-else-if="store.leftTree === 'tasks'" class="lp-scroll">
      <!-- 任务链过滤 -->
      <div class="tl-chains">
        <button class="chain-pill" :class="{ active: !taskChainFilter }" @click="taskChainFilter = null">全部</button>
        <button
          v-for="ch in store.novel?.chains || []"
          :key="ch.id"
          class="chain-pill"
          :class="{ active: taskChainFilter === ch.id }"
          @click="taskChainFilter = ch.id"
        >{{ ch.name }}</button>
        <button class="chain-pill add" title="新建任务链" @click="newChain">＋链</button>
      </div>

      <!-- 按角色分组 -->
      <template v-for="grp in taskGroups" :key="grp.key">
        <div class="count-head">
          <span class="tg-who">{{ grp.icon }} {{ grp.name }}</span>
          <span class="tg-stat">{{ grp.tasks.filter(t=>t.status===2).length }}/{{ grp.tasks.length }} 完成</span>
        </div>
        <button
          v-for="t in grp.tasks"
          :key="t.id"
          class="row task-row"
          :class="{ done: t.status === 2, dark: t.is_public === false }"
          @click="selectTask(t)"
          @contextmenu.prevent="taskMenu($event, t)"
        >
          <span class="tk-state" :class="'s' + t.status">{{ ["○", "◐", "●"][t.status] || "○" }}</span>
          <span class="row-name">{{ t.title }}</span>
          <span class="tk-dark" v-if="t.is_public === false" title="暗线（读者未知）">暗</span>
          <span class="tk-ch" v-if="chapterTitleOf(t.chapter_id)" :title="'关联：' + chapterTitleOf(t.chapter_id)">章</span>
        </button>
      </template>

      <div v-if="!taskGroups.length" class="lp-note">
        {{ taskChainFilter ? "该任务链下暂无任务" : "还没有任务线<br />点右上 ＋ 为角色新建任务/目标" }}
      </div>
    </div>

    <!-- ===== 4. 世界观树（含地图） ===== -->
    <div v-else class="lp-scroll">
      <div v-for="grp in locGroups" :key="grp.kind">
        <div class="count-head">{{ grp.kind }}（{{ grp.list.length }}）</div>
        <TreeNode v-for="l in grp.list" :key="l.id" :node="toNode(l)" :depth="0" kind="location" />
      </div>
      <div v-if="!(store.novel?.locations || []).length" class="lp-note">还没有地点设定，点右上 ＋ 新建</div>
      <div class="lp-foot" @click="store.rightTool = 'map'; store.rightOpen = true">🗺 打开地图工具</div>
    </div>
  </aside>
</template>

<script setup>
import { ref, computed, h } from "vue";
import { store, openTab, allChapters, toast } from "../store";
import { api } from "../api";

const filter = ref("");
const volOpen = ref({});

const treeTabs = [
  { id: "chapters", icon: "📑", name: "章节", tip: "目录：卷 / 章层级", count: () => allChapters(store.novel).length || null },
  { id: "outline", icon: "🗂️", name: "大纲", tip: "大纲：卷/章/节/要点", count: () => (store.novel?.outline || []).length || null },
  { id: "characters", icon: "👥", name: "人物", tip: "人物：按定位分组", count: () => (store.novel?.characters || []).length || null },
  { id: "world", icon: "🗺️", name: "世界", tip: "世界观：地点层级与地图", count: () => (store.novel?.locations || []).length || null },
  { id: "tasks", icon: "🎯", name: "任务线", tip: "角色的任务与目标（非作者待办）", count: () => (store.novel?.tasks || []).filter((t) => t.status < 2).length || null },
];
const treeLabel = computed(() => treeTabs.find((t) => t.id === store.leftTree)?.tip || "");
const addTip = computed(() => ({
  chapters: "新建章节",
  outline: "新建大纲节点",
  characters: "新建人物",
  world: "新建地点",
  tasks: "新建任务（角色目标）",
}[store.leftTree]));

function addCurrent() {
  const t = store.leftTree;
  if (t === "chapters") {
    store.dialog = { kind: "newChapter", payload: { volId: store.novel?.volumes?.[0]?.id } };
  } else if (t === "outline") {
    store.novel.outline.push(mkNode("卷"));
    api.saveNovel(store.novel);
  } else if (t === "characters") {
    const c = { id: uid("c"), name: "新人物", role: "", appearance: "", personality: "", background: "", goals: "", notes: "", relationships: [] };
    store.novel.characters.push(c);
    store.selChar = c.id;
    api.saveNovel(store.novel);
  } else if (t === "world") {
    const l = { id: uid("l"), name: "新地点", kind: "城市", parent_id: null, description: "" };
    store.novel.locations.push(l);
    store.selLoc = l.id;
    api.saveNovel(store.novel);
  } else if (t === "tasks") {
    // 新建任务：优先归属当前选中人物
    const t2 = {
      id: uid("t"),
      title: "新任务",
      description: "",
      status: 0,
      chain_id: null,
      character_id: store.selChar || null,
      chapter_id: store.activeTab || null,
      is_public: true,
      order: (store.novel.tasks || []).length,
    };
    store.novel.tasks = store.novel.tasks || [];
    store.novel.tasks.push(t2);
    api.saveNovel(store.novel);
    selectTask(t2);
  }
}
const uid = (p) => p + Math.random().toString(36).slice(2, 8);
const mkNode = (kind) => ({ id: uid("o"), title: "新节点", kind, content: "", children: [] });

// ---- 章节 ----
const visibleVolumes = computed(() => {
  const vols = store.novel?.volumes || [];
  const q = filter.value.trim();
  if (!q || store.leftTree !== "chapters") return vols;
  return vols
    .map((v) => ({ ...v, chapters: (v.chapters || []).filter((c) => c.title.includes(q)) }))
    .filter((v) => v.chapters.length);
});
function toggleVol(id) {
  volOpen.value[id] = volOpen.value[id] === false;
}
const volWords = (vol) => (vol.chapters || []).reduce((a, c) => a + (c.words || 0), 0);

function chapterMenu(e, c) {
  menu(e, [
    { label: "重命名", run: () => (store.dialog = { kind: "renameChapter", payload: { cid: c.id } }) },
    { label: "上移", run: () => moveChapter(c.id, -1) },
    { label: "下移", run: () => moveChapter(c.id, 1) },
    { label: "删除", danger: true, run: () => (store.dialog = { kind: "deleteChapter", payload: { cid: c.id, title: c.title } }) },
  ]);
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

// ---- 人物 ----
const appearedIds = computed(() => {
  const text = store.chapters[store.activeTab] || "";
  const ids = new Set();
  for (const c of store.novel?.characters || []) {
    if (c.name.length >= 2 && text.includes(c.name)) ids.add(c.id);
  }
  return ids;
});
const charGroups = computed(() => {
  const list = (store.novel?.characters || []).filter((c) => !filter.value.trim() || c.name.includes(filter.value.trim()));
  const order = ["主角", "重要配角", "配角", "反派", "其他", ""];
  const map = new Map();
  for (const c of list) {
    const k = order.includes(c.role) ? c.role : "其他";
    if (!map.has(k)) map.set(k, []);
    map.get(k).push(c);
  }
  return order.filter((r) => map.has(r)).map((role) => ({ role: role || "未分类", list: map.get(role) }));
});
function selectChar(id) {
  store.selChar = id;
  store.fullView = null;
}
function charMenu(e, c) {
  menu(e, [
    { label: "编辑", run: () => selectChar(c.id) },
    { label: "删除", danger: true, run: () => {
        store.novel.characters = store.novel.characters.filter((x) => x.id !== c.id);
        if (store.selChar === c.id) store.selChar = null;
        api.saveNovel(store.novel);
      } },
  ]);
}

// ---- 任务线（角色的任务）----
const taskChainFilter = ref(null);
const taskGroups = computed(() => {
  const tasks = (store.novel?.tasks || []).filter((t) => !taskChainFilter.value || t.chain_id === taskChainFilter.value);
  const groups = new Map();
  for (const t of tasks) {
    const key = t.character_id || "__none__";
    if (!groups.has(key)) groups.set(key, []);
    groups.get(key).push(t);
  }
  const out = [];
  // 先出有归属角色的分组
  for (const c of store.novel?.characters || []) {
    if (groups.has(c.id)) {
      out.push({ key: c.id, name: c.name, icon: "👤", tasks: groups.get(c.id).sort((a, b) => (a.order || 0) - (b.order || 0)) });
      groups.delete(c.id);
    }
  }
  if (groups.has("__none__")) {
    out.push({ key: "__none__", name: "未归属角色", icon: "❓", tasks: groups.get("__none__") });
  }
  return out;
});
function chapterTitleOf(cid) {
  if (!cid) return "";
  return allChapters(store.novel).find((c) => c.id === cid)?.title || "";
}
function selectTask(t) {
  store.selTask = t.id;
  store.rightTool = "tasks";
  store.rightOpen = true;
}
function taskMenu(e, t) {
  const chars = store.novel?.characters || [];
  const chaps = allChapters(store.novel);
  menu(e, [
    { label: "编辑（在右栏）", run: () => selectTask(t) },
    { label: t.is_public === false ? "标记为公开" : "标记为暗线", run: () => { t.is_public = t.is_public === false; api.saveNovel(store.novel); } },
    ...chars.slice(0, 6).map((c) => ({
      label: "归属 → " + c.name,
      run: () => { t.character_id = c.id; api.saveNovel(store.novel); },
    })),
    ...chaps.slice(0, 6).map((c) => ({
      label: "关联 → " + c.title.slice(0, 12),
      run: () => { t.chapter_id = c.id; api.saveNovel(store.novel); },
    })),
    { label: "删除任务", danger: true, run: () => {
        store.novel.tasks = store.novel.tasks.filter((x) => x.id !== t.id);
        api.saveNovel(store.novel);
      } },
  ]);
}
function newChain() {
  const ch = { id: "h" + Math.random().toString(36).slice(2, 8), name: "新任务链", description: "", task_ids: [] };
  store.novel.chains = store.novel.chains || [];
  store.novel.chains.push(ch);
  api.saveNovel(store.novel);
  toast("已新建任务链");
}

// ---- 世界观 ----
const locGroups = computed(() => {
  const list = store.novel?.locations || [];
  const order = ["国家", "城市", "地区", "建筑", "异界", "其他"];
  const map = new Map();
  for (const l of list.filter((x) => !x.parent_id)) {
    const k = order.includes(l.kind) ? l.kind : "其他";
    if (!map.has(k)) map.set(k, []);
    map.get(k).push(l);
  }
  return order.filter((k) => map.has(k)).map((kind) => ({ kind, list: map.get(kind) }));
});
function toNode(loc) {
  const children = (store.novel?.locations || []).filter((x) => x.parent_id === loc.id);
  return {
    id: loc.id,
    title: loc.name,
    kind: "location",
    raw: loc,
    children: children.map(toNode),
  };
}

// ---- 通用菜单 ----
function menu(e, items) {
  window.dispatchEvent(new CustomEvent("jinshu:contextmenu", { detail: { x: e.clientX, y: e.clientY, items } }));
}

/* ---- 通用递归树 ---- */
const TreeNode = {
  props: { node: Object, depth: Number, kind: String },
  setup(props) {
    return () => {
      const n = props.node;
      const icon = n.kind === "卷" ? "📘" : n.kind === "章" ? "📄" : n.kind === "location" ? "📍" : "·";
      const isCur = allChapters(store.novel).some((c) => c.id === store.activeTab && c.title === n.title);
      const click = () => {
        if (props.kind === "location") {
          store.selLoc = n.id;
        } else {
          store.selOutline = n.id;
        }
        store.fullView = null;
      };
      return h("div", {}, [
        h(
          "button",
          {
            class: ["row", "tree-row", { active: n.id === store.selOutline || n.id === store.selLoc, cur: isCur }],
            style: { paddingLeft: 6 + (props.depth || 0) * 13 + "px" },
            onClick: click,
            onContextmenu: (e) => {
              e.preventDefault();
              props.kind === "location"
                ? menu(e, [
                    { label: "添加子地点", run: () => addSubLoc(n) },
                    { label: "删除", danger: true, run: () => delLoc(n.id) },
                  ])
                : menu(e, [
                    { label: "添加子节点", run: () => addChild(n) },
                    { label: "删除", danger: true, run: () => delOutline(n.id) },
                  ]);
            },
          },
          [
            h("span", { class: "tree-icon" }, icon),
            h("span", { class: "row-name" }, n.title),
            isCur ? h("span", { class: "cur-badge" }, "本章") : null,
          ]
        ),
        (n.children || []).length
          ? h("div", {}, n.children.map((c) => h(TreeNode, { node: c, depth: (props.depth || 0) + 1, kind: props.kind })))
          : null,
      ]);
    };
  },
};
function addChild(node) {
  node.children = node.children || [];
  node.children.push(mkNode("节"));
  api.saveNovel(store.novel);
}
function delOutline(id) {
  const rec = (arr) => {
    const i = arr.findIndex((x) => x.id === id);
    if (i >= 0) { arr.splice(i, 1); return true; }
    for (const x of arr) if (x.children && rec(x.children)) return true;
    return false;
  };
  if (rec(store.novel.outline)) {
    if (store.selOutline === id) store.selOutline = null;
    api.saveNovel(store.novel);
  }
}
function addSubLoc(n) {
  store.novel.locations.push({ id: uid("l"), name: "子地点", kind: "建筑", parent_id: n.id, description: "" });
  api.saveNovel(store.novel);
}
function delLoc(id) {
  store.novel.locations = store.novel.locations.filter((x) => x.id !== id && x.parent_id !== id);
  if (store.selLoc === id) store.selLoc = null;
  api.saveNovel(store.novel);
}
</script>

<style scoped>
.left-pane {
  flex-shrink: 0;
  background: var(--panel);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
}
/* 4 树 Tab */
.lp-tabs {
  display: flex;
  flex-shrink: 0;
  border-bottom: 1px solid var(--border);
  padding: 0 3px;
}
.lp-tab {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1px;
  padding: 6px 2px 4px;
  border: none;
  background: transparent;
  color: var(--text-3);
  cursor: pointer;
  font-family: inherit;
  border-bottom: 2px solid transparent;
  position: relative;
  transition: color 0.12s, background 0.12s;
}
.lp-tab:hover { background: var(--hover); color: var(--text); }
.lp-tab.active { color: var(--accent); border-bottom-color: var(--accent); }
.lt-icon { font-size: 14px; }
.lt-name { font-size: 10.5px; }
.lt-count {
  position: absolute;
  top: 2px;
  right: 4px;
  font-size: 9px;
  background: var(--panel-alt);
  color: var(--text-3);
  padding: 0 3px;
  border-radius: 6px;
}
.lp-ops {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 6px 8px 6px 10px;
  flex-shrink: 0;
}
.lp-search {
  flex: 1;
  min-width: 0;
  padding: 4px 9px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--panel-alt);
  color: var(--text);
  font-size: 11.5px;
  outline: none;
  font-family: inherit;
}
.lp-search:focus { border-color: var(--accent); }
.lp-label { flex: 1; font-size: 10.5px; color: var(--text-3); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.lp-scroll { flex: 1; overflow-y: auto; padding: 2px 6px 10px; }
.indent { padding-left: 10px; }
.collapse {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 4px 6px;
  border-radius: 5px;
  cursor: pointer;
  font-size: 12.5px;
  color: var(--text);
}
.collapse:hover { background: var(--hover); }
.collapse .arrow { font-size: 9px; color: var(--text-3); transition: transform 0.15s; }
.collapse.open .arrow { transform: rotate(90deg); }
.vol-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.meta, .row-num { font-size: 10px; color: var(--text-3); flex-shrink: 0; }
.row {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 100%;
  padding: 4px 7px;
  border: none;
  background: transparent;
  border-radius: 5px;
  cursor: pointer;
  font-family: inherit;
  font-size: 12.5px;
  color: var(--text-2);
  text-align: left;
}
.row:hover { background: var(--hover); color: var(--text); }
.row.active { background: var(--accent-soft); color: var(--text); }
.row.cur { border-left: 2px solid var(--accent); }
.row-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; min-width: 0; }
.ch-dot { color: var(--warn); font-size: 8px; flex-shrink: 0; }
.tree-icon { font-size: 11px; flex-shrink: 0; }
.cur-badge { font-size: 9px; color: var(--accent); background: var(--accent-soft); padding: 0 4px; border-radius: 4px; flex-shrink: 0; }
.count-head {
  font-size: 10.5px;
  color: var(--text-3);
  padding: 7px 8px 3px;
  letter-spacing: 0.03em;
}
.p-avatar {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: var(--accent-soft);
  color: var(--accent);
  font-size: 10.5px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.p-badge {
  font-size: 9px;
  color: var(--accent);
  border: 1px solid var(--accent);
  border-radius: 3px;
  padding: 0 3px;
  flex-shrink: 0;
}
/* 任务线 */
.tl-chains {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  padding: 2px 8px 6px;
}
.chain-pill {
  border: 1px solid var(--border);
  background: var(--panel-alt);
  color: var(--text-3);
  font-size: 10.5px;
  padding: 2px 9px;
  border-radius: 10px;
  cursor: pointer;
  font-family: inherit;
  transition: all 0.12s;
}
.chain-pill:hover { color: var(--text); border-color: var(--border-strong); }
.chain-pill.active { background: var(--accent-soft); border-color: var(--accent); color: var(--accent); }
.chain-pill.add { border-style: dashed; padding: 2px 7px; }
.tg-who { color: var(--text-2); }
.tg-stat { float: right; }
.task-row .tk-state { font-size: 11px; flex-shrink: 0; width: 12px; text-align: center; }
.task-row .tk-state.s0 { color: var(--text-3); }
.task-row .tk-state.s1 { color: var(--warn); }
.task-row .tk-state.s2 { color: var(--ok); }
.task-row.done .row-name { color: var(--text-3); text-decoration: line-through; }
.task-row.dark .row-name { font-style: italic; }
.tk-dark {
  font-size: 9px;
  color: var(--purple);
  border: 1px solid var(--purple);
  border-radius: 3px;
  padding: 0 3px;
  flex-shrink: 0;
}
.tk-ch {
  font-size: 9px;
  color: var(--text-3);
  border: 1px solid var(--border);
  border-radius: 3px;
  padding: 0 3px;
  flex-shrink: 0;
}
.lp-note { padding: 14px 12px; font-size: 11.5px; color: var(--text-3); line-height: 1.9; text-align: center; }
.lp-foot {
  margin: 8px 10px 4px;
  padding: 5px;
  text-align: center;
  font-size: 11.5px;
  color: var(--text-3);
  border: 1px dashed var(--border);
  border-radius: 6px;
  cursor: pointer;
}
.lp-foot:hover { color: var(--accent); border-color: var(--accent); }
</style>
