<template>
  <aside class="right-pane" :style="{ width: store.rightWidth + 'px' }">
    <!-- 工具格：未选中具体工具时显示网格 -->
    <div v-if="!store.rightTool" class="tool-grid">
      <div class="tg-head">创意工具</div>
      <div class="tg-items">
        <button v-for="t in tools" :key="t.id" class="tg-card" @click="store.rightTool = t.id">
          <span class="tg-icon">{{ t.icon }}</span>
          <span class="tg-main">
            <span class="tg-name">{{ t.name }}</span>
            <span class="tg-desc">{{ t.desc }}</span>
          </span>
          <span class="tg-badge" v-if="t.badge && t.badge()">{{ t.badge() }}</span>
        </button>
      </div>
      <div class="tg-tip">
        点选工具在此展开，写作时不离开正文
      </div>
    </div>

    <!-- 已选中工具：顶部返回 + 工具内容 -->
    <template v-else>
      <div class="rt-head">
        <button class="icon-btn" title="返回工具格" @click="store.rightTool = null">←</button>
        <span class="rt-title">{{ currentTool?.icon }} {{ currentTool?.name }}</span>
        <button class="icon-btn" title="关闭右栏" @click="store.rightOpen = false">✕</button>
      </div>

      <!-- ===== 起名机 ===== -->
      <div v-if="store.rightTool === 'namer'" class="rt-body">
        <div class="nm-controls">
          <div class="nm-row">
            <button v-for="t in namerTypes" :key="t.id" class="pill" :class="{ active: namer.type === t.id }" @click="namer.type = t.id">{{ t.label }}</button>
          </div>
          <div class="nm-row">
            <button v-for="s in STYLES" :key="s.id" class="pill" :class="{ active: namer.style === s.id }" @click="namer.style = s.id">{{ s.label }}</button>
          </div>
          <div class="nm-row" v-if="namer.type === 'person'">
            <button v-for="g in GENDERS" :key="g" class="pill" :class="{ active: namer.gender === g }" @click="namer.gender = g">{{ g }}</button>
          </div>
          <div class="nm-actions">
            <button class="btn primary" style="flex:1" @click="generate">🎲 生成名字</button>
            <select class="nm-count" v-model.number="namer.count">
              <option :value="8">8 个</option>
              <option :value="16">16 个</option>
              <option :value="24">24 个</option>
            </select>
          </div>
        </div>
        <div class="nm-list">
          <div v-for="(item, i) in namerResults" :key="i" class="nm-item">
            <div class="nm-main">
              <span class="nm-name">{{ item.name }}</span>
              <span class="nm-mean">{{ item.meaning }}</span>
            </div>
            <div class="nm-ops">
              <button class="icon-btn" title="复制" @click="copyName(item.name)">📋</button>
              <button class="icon-btn" title="插入正文" @click="insertName(item.name)">↳</button>
            </div>
          </div>
          <div v-if="!namerResults.length" class="rt-note">
            <span style="font-size:22px;display:block;margin-bottom:6px">🎲</span>
            用本地算法起名，不消耗任何 AI 额度<br />
            支持复姓名（慕容/上官/欧阳…）<br /><br />
            选好类型与风格后点上方按钮
          </div>
        </div>
      </div>

      <!-- ===== 地图 ===== -->
      <div v-else-if="store.rightTool === 'map'" class="rt-body">
        <div class="mp-canvas" @mousemove="onMapMove" @mouseup="mapDrag = null" @mouseleave="mapDrag = null">
          <svg :width="MAPW" :height="MAPH" class="mp-svg">
            <line
              v-for="(e, i) in mapEdges" :key="i"
              :x1="e.a.x" :y1="e.a.y" :x2="e.b.x" :y2="e.b.y"
              stroke="var(--border-strong)" stroke-width="1.2" stroke-dasharray="3 3"
            />
          </svg>
          <div
            v-for="l in store.novel?.locations || []"
            :key="l.id"
            class="mnode"
            :class="{ sel: l.id === store.selLoc }"
            :style="{ left: mapPos(l).x + 'px', top: mapPos(l).y + 'px' }"
            @mousedown.prevent="startMapDrag(l.id, $event)"
            @click="store.selLoc = l.id"
          >
            <span class="mn-icon">{{ locIcon(l.kind) }}</span>
            <span class="mn-name">{{ l.name }}</span>
          </div>
          <div v-if="!(store.novel?.locations || []).length" class="rt-note" style="position:absolute;top:40%;width:100%">
            还没有地点<br />在左栏「世界」中点 ＋ 新建
          </div>
        </div>
        <div v-if="selLoc" class="mp-detail">
          <input class="mp-name" v-model="selLoc.name" @change="save" />
          <select class="mp-select" v-model="selLoc.kind" @change="save">
            <option v-for="k in ['国家','城市','地区','建筑','异界','其他']" :key="k" :value="k">{{ k }}</option>
          </select>
          <textarea class="mp-desc" rows="3" v-model="selLoc.description" placeholder="地点描述…" @change="save"></textarea>
        </div>
      </div>

      <!-- ===== 关系网 ===== -->
      <div v-else-if="store.rightTool === 'graph'" class="rt-body">
        <div class="gr-canvas" @mousemove="onGraphMove" @mouseup="graphDrag = null" @mouseleave="graphDrag = null">
          <svg :width="GW" :height="GH" class="mp-svg">
            <line
              v-for="(e, i) in graphEdges" :key="i"
              :x1="e.a.x" :y1="e.a.y" :x2="e.b.x" :y2="e.b.y"
              stroke="var(--accent)" stroke-width="1.3" opacity="0.6"
            />
            <text
              v-for="(e, i) in graphEdges" :key="'t'+i"
              :x="(e.a.x+e.b.x)/2" :y="(e.a.y+e.b.y)/2 - 6"
              text-anchor="middle" font-size="9" fill="var(--text-3)"
            >{{ e.relation }}</text>
          </svg>
          <div
            v-for="c in store.novel?.characters || []"
            :key="c.id"
            class="gnode2"
            :class="{ sel: c.id === store.selChar }"
            :style="{ left: gpos(c.id).x + 'px', top: gpos(c.id).y + 'px' }"
            @mousedown.prevent="startGraphDrag(c.id, $event)"
            @click="store.selChar = c.id"
          >{{ c.name }}</div>
          <div v-if="!(store.novel?.characters || []).length" class="rt-note" style="position:absolute;top:40%;width:100%">
            还没有人物
          </div>
        </div>
        <div class="gr-ops">
          <button class="btn sm" @click="store.canvasPos = {}">重置布局</button>
          <span class="rt-hint">拖动节点调整</span>
        </div>
      </div>

      <!-- ===== 时间轴 ===== -->
      <div v-else-if="store.rightTool === 'timeline'" class="rt-body">
        <div class="tl-toolbar">
          <span class="rt-hint">{{ (store.novel?.timeline || []).length }} 个事件</span>
          <button class="btn sm" @click="newEvent">＋ 新建</button>
        </div>
        <div class="tl-scroll">
          <div
            v-for="(e, i) in sortedEvents" :key="e.id"
            class="tl-item"
            :class="{ related: e.chapter_id === store.activeTab }"
          >
            <div class="tl-head">
              <span class="tl-dot" :class="{ now: e.chapter_id === store.activeTab }"></span>
              <input class="tl-time" v-model="e.time" @change="save" />
              <button class="icon-btn" :title="e.chapter_id === store.activeTab ? '取消关联本章' : '关联本章'" @click="linkToCurrent(e)">
                {{ e.chapter_id === store.activeTab ? "✓" : "↳" }}
              </button>
              <button class="icon-btn danger" @click="delEvent(i)">🗑</button>
            </div>
            <input class="tl-title" v-model="e.title" @change="save" />
            <textarea class="tl-desc" rows="2" v-model="e.description" placeholder="事件描述…" @change="save"></textarea>
          </div>
          <div v-if="!(store.novel?.timeline || []).length" class="rt-note">还没有事件<br />点上方 ＋ 记录剧情节点</div>
        </div>
      </div>

      <!-- ===== 任务 ===== -->
      <div v-else-if="store.rightTool === 'tasks'" class="rt-body">
        <div class="tl-toolbar">
          <span class="rt-hint">未开始 {{ counts[0] }} · 进行 {{ counts[1] }} · 完成 {{ counts[2] }}</span>
          <button class="btn sm" @click="newTask">＋ 新建任务</button>
        </div>
        <div class="tl-scroll">
          <div v-for="(col, ci) in taskCols" :key="ci">
            <div class="tk-head" :style="{ color: col.color }">{{ col.name }}（{{ byStatus[ci].length }}）</div>
            <div
              v-for="t in byStatus[ci]" :key="t.id"
              class="tk-card"
              :class="{ dark: t.is_public === false, sel: t.id === store.selTask }"
              draggable="true"
              @dragstart="dragTask = t.id"
              @dragover.prevent
              @drop.prevent="dropTask(ci)"
              @click="store.selTask = t.id"
            >
              <div class="tk-top">
                <input class="tk-title" v-model="t.title" @change="save" />
                <button class="icon-btn" :title="t.is_public === false ? '暗线（读者未知）· 点击改为公开' : '公开 · 点击改为暗线'"
                        @click.stop="t.is_public = t.is_public === false; save()">
                  {{ t.is_public === false ? "🌑" : "☀" }}
                </button>
              </div>
              <textarea class="tk-desc" rows="1" v-model="t.description" placeholder="目标说明…" @change="save"></textarea>
              <!-- 归属角色 / 关联章节 -->
              <div class="tk-links">
                <select class="tk-select" v-model="t.character_id" @change="save" :title="'归属角色'">
                  <option :value="null">未归属角色</option>
                  <option v-for="c in (store.novel?.characters || [])" :key="c.id" :value="c.id">{{ c.name }}</option>
                </select>
                <select class="tk-select" v-model="t.chapter_id" @change="save" :title="'关联章节'">
                  <option :value="null">不关联章节</option>
                  <option v-for="c in allChapters(store.novel)" :key="c.id" :value="c.id">{{ c.title }}</option>
                </select>
              </div>
              <div class="tk-ops">
                <button v-if="ci > 0" class="icon-btn" title="退一步" @click="t.status = ci - 1; save()">◀</button>
                <button v-if="ci < 2" class="icon-btn" title="推进一步" @click="t.status = ci + 1; save()">▶</button>
                <button class="icon-btn danger" style="margin-left:auto" @click="delTask(t.id)">🗑</button>
              </div>
            </div>
            <div v-if="!byStatus[ci].length" class="tk-empty">（空）</div>
          </div>
        </div>
      </div>
    </template>
  </aside>
</template>

<script setup>
import { ref, computed } from "vue";
import { store, allChapters, openTab, toast } from "../store";
import { api } from "../api";
import { generateNames, STYLES, GENDERS } from "../names";

const MAPW = 900, MAPH = 700;
const GW = 900, GH = 700;

const tools = [
  { id: "namer", icon: "🎲", name: "起名机", desc: "人物/书名/地名" },
  { id: "map", icon: "🗺", name: "地图", desc: "地点分布画布", badge: () => (store.novel?.locations || []).length || null },
  { id: "graph", icon: "🧩", name: "关系网", desc: "人物关系图谱", badge: () => (store.novel?.characters || []).length || null },
  { id: "timeline", icon: "⏱", name: "时间轴", desc: "剧情事件脉络", badge: () => (store.novel?.timeline || []).length || null },
  { id: "tasks", icon: "🎯", name: "任务线", desc: "角色任务与目标", badge: () => (store.novel?.tasks || []).filter((t) => t.status < 2).length || null },
];
const currentTool = computed(() => tools.find((t) => t.id === store.rightTool));

function save() {
  api.saveNovel(store.novel);
}

/* ---- 起名机 ---- */
const namerTypes = [
  { id: "person", label: "人物名" },
  { id: "book", label: "书名" },
  { id: "place", label: "地名" },
];
const namer = ref({ type: "person", style: "gufeng", gender: "中性", count: 8 });
const namerResults = ref([]);
function existingNames() {
  const n = store.novel;
  if (!n) return [];
  if (namer.value.type === "person") return (n.characters || []).map((c) => c.name);
  if (namer.value.type === "book") return n.meta.title ? [n.meta.title] : [];
  return (n.locations || []).map((l) => l.name);
}
function generate() {
  namerResults.value = generateNames({
    type: namer.value.type,
    style: namer.value.style,
    gender: namer.value.gender,
    count: namer.value.count,
    exclude: existingNames(),
  });
}
function copyName(name) {
  navigator.clipboard.writeText(name);
  toast("已复制：" + name);
}
function insertName(name) {
  const cid = store.activeTab;
  if (!cid) return toast("请先打开一个章节", false);
  window.dispatchEvent(new CustomEvent("jinshu:insert", { detail: { cid, content: name } }));
  toast("已插入正文");
}

/* ---- 地图 ---- */
const mapDrag = ref(null);
function mapPos(l) {
  if (!store.mapPos) store.mapPos = {};
  if (!store.mapPos[l.id]) {
    const list = store.novel?.locations || [];
    const i = list.findIndex((x) => x.id === l.id);
    const n = Math.max(1, list.length);
    const angle = (Math.PI * 2 * i) / n;
    const r = Math.min(MAPW, MAPH) * 0.3;
    store.mapPos[l.id] = { x: MAPW / 2 + r * Math.cos(angle), y: MAPH / 2 + r * Math.sin(angle) };
  }
  return store.mapPos[l.id];
}
function startMapDrag(id, e) {
  const rect = e.currentTarget.parentElement.getBoundingClientRect();
  mapDrag.value = { id, rect };
}
function onMapMove(e) {
  if (!mapDrag.value) return;
  const { id, rect } = mapDrag.value;
  const p = store.mapPos[id];
  p.x = Math.max(6, Math.min(MAPW - 100, e.clientX - rect.left - 40));
  p.y = Math.max(6, Math.min(MAPH - 36, e.clientY - rect.top - 14));
}
const mapEdges = computed(() => {
  const out = [];
  for (const l of store.novel?.locations || []) {
    if (l.parent_id) {
      const p = (store.novel.locations || []).find((x) => x.id === l.parent_id);
      if (p) out.push({ a: mapPos(p), b: mapPos(l) });
    }
  }
  return out;
});
const locIcon = (k) => ({ 国家: "🏳️", 城市: "🏙️", 地区: "🏞️", 建筑: "🏛️", 异界: "🌌" }[k] || "📍");
const selLoc = computed(() => (store.novel?.locations || []).find((l) => l.id === store.selLoc) || null);

/* ---- 关系网 ---- */
const graphDrag = ref(null);
function gpos(id) {
  if (!store.canvasPos) store.canvasPos = {};
  if (!store.canvasPos[id]) {
    const list = store.novel?.characters || [];
    const i = list.findIndex((x) => x.id === id);
    const n = Math.max(1, list.length);
    const angle = (Math.PI * 2 * i) / n - Math.PI / 2;
    const r = Math.min(GW, GH) * 0.32;
    store.canvasPos[id] = { x: GW / 2 + r * Math.cos(angle), y: GH / 2 + r * Math.sin(angle) };
  }
  return store.canvasPos[id];
}
function startGraphDrag(id, e) {
  const rect = e.currentTarget.parentElement.getBoundingClientRect();
  graphDrag.value = { id, rect };
}
function onGraphMove(e) {
  if (!graphDrag.value) return;
  const { id, rect } = graphDrag.value;
  const p = store.canvasPos[id];
  p.x = Math.max(6, Math.min(GW - 90, e.clientX - rect.left - 32));
  p.y = Math.max(6, Math.min(GH - 36, e.clientY - rect.top - 14));
}
const graphEdges = computed(() => {
  const out = [];
  for (const c of store.novel?.characters || []) {
    for (const r of c.relationships || []) {
      const b = (store.novel.characters || []).find((x) => x.id === r.target_id);
      if (b) out.push({ a: gpos(c.id), b: gpos(b.id), relation: r.relation });
    }
  }
  return out;
});

/* ---- 时间轴 ---- */
const sortedEvents = computed(() =>
  [...(store.novel?.timeline || [])].sort((a, b) => String(a.time || "").localeCompare(String(b.time || "")))
);
function newEvent() {
  store.novel.timeline.push({
    id: "e" + Math.random().toString(36).slice(2, 8),
    title: "新事件",
    time: "第1卷",
    description: "",
    character_ids: [],
    location_id: null,
    chapter_id: store.activeTab,
  });
  save();
}
function linkToCurrent(e) {
  e.chapter_id = e.chapter_id === store.activeTab ? null : store.activeTab;
  save();
}
function delEvent(i) {
  store.novel.timeline.splice(i, 1);
  save();
}

/* ---- 任务 ---- */
const taskCols = [
  { name: "待办", color: "var(--text-2)" },
  { name: "进行中", color: "var(--warn)" },
  { name: "已完成", color: "var(--ok)" },
];
const byStatus = computed(() => [0, 1, 2].map((s) => (store.novel?.tasks || []).filter((t) => t.status === s)));
const counts = computed(() => byStatus.value.map((a) => a.length));
const dragTask = ref(null);
function newTask() {
  const t = {
    id: "t" + Math.random().toString(36).slice(2, 8),
    title: "新任务",
    description: "",
    status: 0,
    chain_id: null,
    character_id: store.selChar || null,
    chapter_id: store.activeTab || null,
    is_public: true,
    order: (store.novel?.tasks || []).length,
  };
  store.novel.tasks = store.novel.tasks || [];
  store.novel.tasks.push(t);
  store.selTask = t.id;
  save();
}
function delTask(id) {
  store.novel.tasks = store.novel.tasks.filter((t) => t.id !== id);
  save();
}
function dropTask(status) {
  const t = (store.novel?.tasks || []).find((x) => x.id === dragTask.value);
  if (t) {
    t.status = status;
    save();
  }
  dragTask.value = null;
}
</script>

<style scoped>
.right-pane {
  flex-shrink: 0;
  background: var(--panel);
  border-left: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
}
/* 工具格 */
.tool-grid { flex: 1; overflow-y: auto; padding: 12px 10px; }
.tg-head { font-size: 12px; color: var(--text-3); padding: 0 4px 8px; letter-spacing: 0.04em; }
.tg-items { display: flex; flex-direction: column; gap: 7px; }
.tg-card {
  position: relative;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--panel-alt);
  color: var(--text);
  cursor: pointer;
  font-family: inherit;
  text-align: left;
  transition: border-color 0.13s, background 0.13s;
}
.tg-card:hover { border-color: var(--accent); background: var(--hover); }
.tg-icon { font-size: 17px; flex-shrink: 0; width: 20px; text-align: center; }
.tg-main { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 1px; }
.tg-name { font-size: 12.5px; font-weight: 600; }
.tg-desc { font-size: 11px; color: var(--text-2); opacity: 0.75; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.tg-badge {
  font-size: 10px;
  color: var(--accent);
  background: var(--accent-soft);
  padding: 1px 6px;
  border-radius: 8px;
  flex-shrink: 0;
}
.tg-tip { margin-top: 14px; padding: 0 4px; font-size: 10.5px; color: var(--text-3); line-height: 1.8; }

/* 工具头部 */
.rt-head {
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 7px 8px 7px 10px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}
.rt-title { flex: 1; font-size: 12.5px; font-weight: 600; color: var(--text); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.rt-body { flex: 1; display: flex; flex-direction: column; min-height: 0; }
.rt-note { padding: 16px 12px; font-size: 11.5px; color: var(--text-3); line-height: 1.9; text-align: center; }
.rt-hint { font-size: 10.5px; color: var(--text-3); }

/* 起名机 */
.nm-controls { padding: 9px 10px; border-bottom: 1px solid var(--border); flex-shrink: 0; }
.nm-row { display: flex; gap: 4px; flex-wrap: wrap; margin-bottom: 5px; }
.nm-actions { display: flex; gap: 5px; margin-top: 7px; }
.nm-count {
  background: var(--panel-alt);
  border: 1px solid var(--border);
  border-radius: 6px;
  color: var(--text);
  font-size: 12px;
  padding: 0 6px;
  outline: none;
  font-family: inherit;
  cursor: pointer;
}
.nm-list { flex: 1; overflow-y: auto; padding: 7px 8px; }
.nm-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 7px 9px;
  border-radius: 7px;
  background: var(--panel-alt);
  margin-bottom: 5px;
}
.nm-main { flex: 1; min-width: 0; }
.nm-name { display: block; font-size: 13.5px; font-weight: 600; color: var(--text); }
.nm-mean { display: block; font-size: 10px; color: var(--text-3); margin-top: 1px; }
.nm-ops { display: flex; gap: 2px; flex-shrink: 0; }

/* 地图 / 关系网 */
.mp-canvas, .gr-canvas {
  flex: 1;
  position: relative;
  overflow: auto;
  background: var(--editor);
  border-bottom: 1px solid var(--border);
  min-height: 0;
}
.mp-svg { position: absolute; inset: 0; pointer-events: none; }
.mnode, .gnode2 {
  position: absolute;
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 5px 11px;
  border-radius: 8px;
  border: 1px solid var(--border-strong);
  background: var(--panel-alt);
  color: var(--text);
  font-size: 11.5px;
  cursor: grab;
  user-select: none;
  white-space: nowrap;
}
.mnode:hover, .gnode2:hover { background: var(--hover); }
.mnode.sel, .gnode2.sel { background: var(--accent-soft); border-color: var(--accent); }
.mn-icon { font-size: 11px; }
.mp-detail {
  flex-shrink: 0;
  padding: 8px 10px;
  border-top: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  gap: 5px;
  max-height: 190px;
  overflow-y: auto;
}
.mp-name, .mp-select {
  background: var(--panel-alt);
  border: 1px solid var(--border);
  border-radius: 5px;
  color: var(--text);
  font-size: 12px;
  padding: 4px 7px;
  outline: none;
  font-family: inherit;
}
.mp-desc {
  background: var(--panel-alt);
  border: 1px solid var(--border);
  border-radius: 5px;
  color: var(--text);
  font-size: 11.5px;
  padding: 5px 7px;
  outline: none;
  resize: vertical;
  font-family: inherit;
  line-height: 1.65;
}
.gr-ops { display: flex; align-items: center; gap: 8px; padding: 7px 10px; flex-shrink: 0; }

/* 时间轴 / 任务 */
.tl-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 7px 10px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}
.tl-scroll { flex: 1; overflow-y: auto; padding: 8px; }
.tl-item {
  padding: 8px 9px;
  border-radius: 8px;
  border: 1px solid transparent;
  background: var(--panel-alt);
  margin-bottom: 6px;
}
.tl-item.related { border-color: var(--accent); background: var(--accent-softer); }
.tl-head { display: flex; align-items: center; gap: 5px; margin-bottom: 3px; }
.tl-dot { width: 7px; height: 7px; border-radius: 50%; background: var(--border-strong); flex-shrink: 0; }
.tl-dot.now { background: var(--accent); }
.tl-time {
  flex: 1;
  background: transparent;
  border: none;
  color: var(--text-3);
  font-size: 10.5px;
  outline: none;
  font-family: inherit;
  min-width: 0;
}
.tl-title {
  width: 100%;
  background: transparent;
  border: none;
  color: var(--text);
  font-size: 12.5px;
  font-weight: 500;
  outline: none;
  font-family: inherit;
  padding: 1px 0;
}
.tl-desc {
  width: 100%;
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: 5px;
  color: var(--text-2);
  font-size: 11px;
  padding: 4px 6px;
  outline: none;
  resize: vertical;
  font-family: inherit;
  margin-top: 3px;
  line-height: 1.6;
}
.tk-head { font-size: 10.5px; padding: 8px 4px 4px; }
.tk-card {
  padding: 7px 8px;
  border-radius: 7px;
  border: 1px solid var(--border);
  background: var(--panel-alt);
  margin-bottom: 5px;
  cursor: grab;
}
.tk-card:active { cursor: grabbing; }
.tk-title {
  width: 100%;
  background: transparent;
  border: none;
  color: var(--text);
  font-size: 12px;
  outline: none;
  font-family: inherit;
}
.tk-desc {
  width: 100%;
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: 5px;
  color: var(--text-2);
  font-size: 10.5px;
  padding: 3px 6px;
  outline: none;
  resize: vertical;
  font-family: inherit;
  margin-top: 4px;
  line-height: 1.6;
}
.tk-top { display: flex; align-items: center; gap: 3px; }
.tk-links { display: flex; gap: 4px; margin-top: 4px; }
.tk-select {
  flex: 1;
  min-width: 0;
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: 5px;
  color: var(--text-2);
  font-size: 10px;
  padding: 2px 4px;
  outline: none;
  font-family: inherit;
  cursor: pointer;
}
.tk-card.dark { border-left: 2px solid var(--purple); }
.tk-card.sel { border-color: var(--accent); }
.tk-ops { display: flex; gap: 2px; margin-top: 4px; }
.tk-empty { font-size: 10.5px; color: var(--text-3); padding: 2px 8px 6px; }
</style>
