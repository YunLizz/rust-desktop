<template>
  <div class="track-wrap" v-if="open">
    <!-- 轨道工具栏 -->
    <div class="tw-bar">
      <span class="tw-title">🎬 剧情时间轴</span>
      <div class="tw-tools">
        <button class="tw-btn" :class="{ on: groupBy === 'chapter' }" @click="groupBy = groupBy === 'chapter' ? 'flat' : 'chapter'">
          {{ groupBy === "chapter" ? "按章节分轨" : "单轨平铺" }}
        </button>
        <span class="tw-sep"></span>
        <button class="tw-btn" title="缩小" @click="zoom(-1)">－</button>
        <span class="tw-zoom">{{ Math.round(scale * 100) }}%</span>
        <button class="tw-btn" title="放大" @click="zoom(1)">＋</button>
        <span class="tw-sep"></span>
        <button class="tw-btn" title="新建事件" @click="addEvent">＋ 事件</button>
        <button class="tw-btn" title="收起时间轴" @click="open = false">▴</button>
      </div>
    </div>

    <!-- 轨道区域 -->
    <div class="tw-body" :style="{ height: bodyH + 'px' }">
      <!-- 左侧事件列表（剪辑软件的素材列） -->
      <div class="tw-names">
        <div class="tw-name-head">事件</div>
        <div
          v-for="(row, ri) in rows"
          :key="ri"
          class="tw-name-row"
          :class="{ head: row.isHead }"
          :style="{ height: row.h + 'px' }"
          :title="row.label"
        >
          <span class="tw-name-txt">{{ row.label }}</span>
        </div>
      </div>

      <!-- 右侧轨道画布 -->
      <div class="tw-canvas" ref="canvas" @wheel.ctrl.prevent="onWheel">
        <!-- 时间刻度 -->
        <div class="tw-ruler" :style="{ width: totalW + 'px' }">
          <div v-for="(m, i) in marks" :key="i" class="tw-mark" :style="{ left: m.x + 'px' }">
            <span class="tw-mark-line"></span>
            <span class="tw-mark-txt">{{ m.label }}</span>
          </div>
        </div>

        <!-- 轨道行 -->
        <div class="tw-rows" :style="{ width: totalW + 'px' }">
          <div
            v-for="(row, ri) in rows"
            :key="ri"
            class="tw-row"
            :class="{ head: row.isHead }"
            :style="{ height: row.h + 'px' }"
          >
            <!-- 事件块 -->
            <div
              v-for="ev in row.events"
              :key="ev.id"
              class="tw-clip"
              :class="[kindClass(ev.kind), { cur: ev.chapter_id === store.activeTab, sel: ev.id === store.selEvent, dark: ev.kind === '暗线' }]"
              :style="{ left: clipX(ev) + 'px', width: clipW(ev) + 'px' }"
              :title="clipTip(ev)"
              @mousedown.prevent="startDrag(ev, $event)"
              @click="store.selEvent = ev.id"
              @dblclick="openInTool(ev)"
            >
              <span class="tw-clip-handle left"></span>
              <span class="tw-clip-name">{{ ev.title }}</span>
              <span class="tw-clip-handle right" @mousedown.stop.prevent="startResize(ev, $event)"></span>
            </div>
          </div>
          <div v-if="!hasEvents" class="tw-empty">
            还没有事件 —— 点「＋ 事件」新建，或按章节分轨后拖动排布
          </div>
        </div>
      </div>
    </div>

    <!-- 选中事件详情条 -->
    <div class="tw-detail" v-if="selEvent">
      <span class="twd-kind" :class="kindClass(selEvent.kind)">{{ selEvent.kind || "主线" }}</span>
      <input class="twd-input" v-model="selEvent.title" @change="save" placeholder="事件名" />
      <input class="twd-input sm" v-model="selEvent.time" @change="save" placeholder="时间/卷章" />
      <select class="twd-select" v-model="selEvent.kind" @change="save">
        <option v-for="k in KINDS" :key="k" :value="k">{{ k }}</option>
      </select>
      <select class="twd-select" v-model="selEvent.chapter_id" @change="save">
        <option :value="null">不关联章节</option>
        <option v-for="c in chapters" :key="c.id" :value="c.id">{{ c.title }}</option>
      </select>
      <button class="tw-btn danger" @click="delSel">🗑 删除</button>
    </div>
  </div>

  <!-- 收起态：只有一条窄触发条 -->
  <button v-else class="track-trigger" title="展开剧情时间轴" @click="open = true">
    🎬 剧情时间轴 <span class="tt-count" v-if="eventCount">{{ eventCount }}</span> ▾
  </button>
</template>

<script setup>
import { ref, computed } from "vue";
import { store, allChapters, openTab, toast } from "../store";
import { api } from "../api";

const KINDS = ["主线", "支线", "暗线", "回忆"];
const PX_PER_UNIT = 120; // 基础单位宽度（缩放前）
const ROW_H = 30;
const HEAD_H = 22;

const scale = ref(1);
const groupBy = ref("chapter");
const drag = ref(null);

const open = computed({
  get: () => store.timelineOpen,
  set: (v) => (store.timelineOpen = v),
});

const chapters = computed(() => allChapters(store.novel));
const events = computed(() => store.novel?.timeline || []);
const eventCount = computed(() => events.value.length);
const hasEvents = computed(() => events.value.length > 0);
const selEvent = computed(() => events.value.find((e) => e.id === store.selEvent) || null);

const unit = computed(() => PX_PER_UNIT * scale.value);
const bodyH = computed(() => Math.min(260, rows.value.reduce((a, r) => a + r.h, 0) + 28 + 20));

// 事件顺序 = order 字段（拖动可改）
function evOrder(e) {
  return e.order ?? 0;
}
function sortByOrder(list) {
  return [...list].sort((a, b) => evOrder(a) - evOrder(b));
}

/* 轨道行：按章节分轨 或 单轨平铺 */
const rows = computed(() => {
  if (groupBy.value === "flat") {
    return [{ isHead: false, label: "全部事件", h: ROW_H + 8, events: sortByOrder(events.value) }];
  }
  // 按章节分轨：每个有关联章节的事件归到该章节轨；未关联的归"未关联"轨
  const map = new Map();
  for (const c of chapters.value) map.set(c.id, []);
  const orphan = [];
  for (const e of events.value) {
    if (e.chapter_id && map.has(e.chapter_id)) map.get(e.chapter_id).push(e);
    else orphan.push(e);
  }
  const out = [];
  for (const c of chapters.value) {
    const list = map.get(c.id);
    if (!list.length) continue;
    out.push({ isHead: true, label: c.title, h: HEAD_H, events: [] });
    out.push({ isHead: false, label: "", h: ROW_H + 6, events: sortByOrder(list) });
  }
  if (orphan.length) {
    out.push({ isHead: true, label: "未关联章节", h: HEAD_H, events: [] });
    out.push({ isHead: false, label: "", h: ROW_H + 6, events: sortByOrder(orphan) });
  }
  return out;
});

/* 横向位置：单轨按序号，分轨按 order */
function clipX(e) {
  return 16 + Math.max(0, evOrder(e)) * unit.value;
}
function clipW(e) {
  // 有时长感：默认 1 个单位宽；若有 duration 字段可用
  return Math.max(56, unit.value - 10);
}

/* 刻度 */
const marks = computed(() => {
  const n = Math.max(6, maxOrder() + 3);
  const out = [];
  for (let i = 0; i < n; i++) {
    out.push({ x: 16 + i * unit.value, label: groupBy.value === "chapter" ? "第" + (i + 1) + "节拍" : "T" + (i + 1) });
  }
  return out;
});
function maxOrder() {
  return events.value.reduce((m, e) => Math.max(m, evOrder(e) || 0), 0);
}
const totalW = computed(() => 16 + (maxOrder() + 3) * unit.value + 80);

/* 交互 */
function zoom(d) {
  scale.value = Math.max(0.4, Math.min(2.4, +(scale.value + d * 0.2).toFixed(2)));
}
function onWheel(ev) {
  zoom(ev.deltaY < 0 ? 1 : -1);
}
function startDrag(e, ev) {
  drag.value = { e, startX: ev.clientX, startOrder: evOrder(e) };
  window.addEventListener("mousemove", onDragMove);
  window.addEventListener("mouseup", endDrag);
}
function onDragMove(ev) {
  if (!drag.value) return;
  const dx = ev.clientX - drag.value.startX;
  const delta = Math.round(dx / unit.value);
  const e = drag.value.e;
  e.order = Math.max(0, drag.value.startOrder + delta);
}
function endDrag() {
  if (drag.value) {
    api.saveNovel(store.novel);
    drag.value = null;
  }
  window.removeEventListener("mousemove", onDragMove);
  window.removeEventListener("mouseup", endDrag);
}
function startResize(e, ev) {
  // 拖动右边缘调整"占据多个节拍"
  const startX = ev.clientX;
  const startSpan = e.span || 1;
  const move = (m) => {
    const delta = Math.round((m.clientX - startX) / unit.value);
    e.span = Math.max(1, startSpan + delta);
    api.saveNovel(store.novel);
  };
  const up = () => {
    window.removeEventListener("mousemove", move);
    window.removeEventListener("mouseup", up);
  };
  window.addEventListener("mousemove", move);
  window.addEventListener("mouseup", up);
}

function addEvent() {
  if (!store.novel) return toast("请先打开一部小说", false);
  const e = {
    id: "e" + Math.random().toString(36).slice(2, 8),
    title: "新事件",
    time: "第1卷",
    description: "",
    character_ids: [],
    location_id: null,
    chapter_id: store.activeTab || null,
    order: maxOrder() + 1,
    kind: "主线",
    span: 1,
  };
  store.novel.timeline = store.novel.timeline || [];
  store.novel.timeline.push(e);
  store.selEvent = e.id;
  save();
  toast("已新建事件（拖动可调整位置）");
}
function delSel() {
  store.novel.timeline = events.value.filter((e) => e.id !== store.selEvent);
  store.selEvent = null;
  save();
}
function openInTool(e) {
  store.selEvent = e.id;
  store.rightTool = "timeline";
  store.rightOpen = true;
}
function save() {
  api.saveNovel(store.novel);
}
function kindClass(k) {
  const m = { 主线: "k-main", 支线: "k-side", 暗线: "k-dark", 回忆: "k-memo" };
  return m[k] || "k-main";
}
function clipTip(e) {
  let t = `${e.title}（${e.kind || "主线"}）`;
  if (e.time) t += ` · ${e.time}`;
  if (e.chapter_id) t += ` · ${chapters.value.find((c) => c.id === e.chapter_id)?.title || ""}`;
  if (e.description) t += `\n${e.description}`;
  return t + "\n\n拖动调整位置 · 拖右边缘调整长度 · 双击在右栏编辑";
}
</script>

<style scoped>
.track-wrap {
  flex-shrink: 0;
  background: var(--chrome);
  border-bottom: 1px solid var(--border);
  display: flex;
  flex-direction: column;
}
/* 工具栏 */
.tw-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 10px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}
.tw-title { font-size: 11.5px; color: var(--text-2); }
.tw-tools { display: flex; align-items: center; gap: 3px; }
.tw-btn {
  border: 1px solid transparent;
  background: transparent;
  color: var(--text-3);
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 5px;
  cursor: pointer;
  font-family: inherit;
  transition: all 0.12s;
}
.tw-btn:hover { background: var(--hover); color: var(--text); }
.tw-btn.on { background: var(--accent-soft); color: var(--accent); }
.tw-btn.danger { color: var(--danger); }
.tw-btn.danger:hover { background: var(--danger-soft); }
.tw-zoom { font-size: 10.5px; color: var(--text-3); min-width: 34px; text-align: center; }
.tw-sep { width: 1px; height: 12px; background: var(--border); margin: 0 3px; }

/* 轨道主体 */
.tw-body {
  display: flex;
  min-height: 0;
  overflow: hidden;
}
/* 素材列 */
.tw-names {
  width: 130px;
  flex-shrink: 0;
  border-right: 1px solid var(--border);
  overflow: hidden;
  background: var(--panel);
}
.tw-name-head {
  height: 22px;
  font-size: 10px;
  color: var(--text-3);
  padding: 4px 8px;
  border-bottom: 1px solid var(--border);
}
.tw-name-row {
  display: flex;
  align-items: center;
  padding: 0 8px;
  font-size: 11px;
  color: var(--text-2);
  border-bottom: 1px solid var(--border);
  overflow: hidden;
}
.tw-name-row.head {
  color: var(--text-3);
  font-size: 10px;
  background: var(--panel-alt);
}
.tw-name-txt { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

/* 轨道画布 */
.tw-canvas {
  flex: 1;
  overflow: auto;
  position: relative;
  background: var(--editor);
}
.tw-ruler {
  height: 22px;
  position: relative;
  border-bottom: 1px solid var(--border);
  min-width: 100%;
}
.tw-mark { position: absolute; top: 0; height: 100%; }
.tw-mark-line { position: absolute; left: 0; top: 12px; bottom: 0; width: 1px; background: var(--border); }
.tw-mark-txt { position: absolute; left: 3px; top: 3px; font-size: 9px; color: var(--text-3); white-space: nowrap; }
.tw-rows { position: relative; min-width: 100%; }
.tw-row {
  position: relative;
  border-bottom: 1px solid var(--border);
}
.tw-row.head {
  background: var(--panel-alt);
  border-bottom-color: var(--border);
}
/* 事件块 */
.tw-clip {
  position: absolute;
  top: 4px;
  height: calc(100% - 10px);
  min-width: 56px;
  border-radius: 5px;
  display: flex;
  align-items: center;
  padding: 0 8px;
  cursor: grab;
  user-select: none;
  font-size: 11px;
  color: #fff;
  overflow: hidden;
  transition: box-shadow 0.12s, filter 0.12s;
  border: 1px solid rgba(255, 255, 255, 0.15);
}
.tw-clip:active { cursor: grabbing; }
.tw-clip:hover { filter: brightness(1.15); box-shadow: 0 2px 10px rgba(0, 0, 0, 0.35); }
.tw-clip.cur { box-shadow: 0 0 0 2px var(--accent); }
.tw-clip.sel { outline: 1.5px solid #fff; outline-offset: -2px; }
.tw-clip-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; flex: 1; }
.tw-clip-handle { width: 4px; flex-shrink: 0; }
.tw-clip-handle.right { cursor: ew-resize; }
/* 类型配色（剪辑软件风格） */
.k-main { background: linear-gradient(180deg, #4a6fd4, #3a5cb8); }
.k-side { background: linear-gradient(180deg, #3fa88a, #2f8a70); }
.k-dark { background: linear-gradient(180deg, #7a5cc4, #6248a8); }
.k-memo { background: linear-gradient(180deg, #c48a3f, #a8702f); }
.tw-empty {
  position: absolute;
  left: 20px;
  top: 40px;
  font-size: 11.5px;
  color: var(--text-3);
}

/* 详情条 */
.tw-detail {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 10px;
  border-top: 1px solid var(--border);
  flex-shrink: 0;
}
.twd-kind {
  font-size: 10px;
  color: #fff;
  padding: 2px 8px;
  border-radius: 8px;
  flex-shrink: 0;
}
.twd-input {
  flex: 1;
  min-width: 0;
  background: var(--panel-alt);
  border: 1px solid var(--border);
  border-radius: 5px;
  color: var(--text);
  font-size: 11.5px;
  padding: 3px 8px;
  outline: none;
  font-family: inherit;
}
.twd-input.sm { flex: 0 0 100px; }
.twd-select {
  background: var(--panel-alt);
  border: 1px solid var(--border);
  border-radius: 5px;
  color: var(--text);
  font-size: 11px;
  padding: 3px 6px;
  outline: none;
  font-family: inherit;
  max-width: 150px;
}

/* 收起态触发条 */
.track-trigger {
  flex-shrink: 0;
  border: none;
  border-bottom: 1px solid var(--border);
  background: var(--chrome);
  color: var(--text-3);
  font-size: 11px;
  padding: 3px 12px;
  cursor: pointer;
  text-align: left;
  font-family: inherit;
  display: flex;
  align-items: center;
  gap: 6px;
}
.track-trigger:hover { color: var(--text); background: var(--hover); }
.tt-count {
  font-size: 9.5px;
  background: var(--accent-soft);
  color: var(--accent);
  padding: 0 5px;
  border-radius: 7px;
}
</style>
