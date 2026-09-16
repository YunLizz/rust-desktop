<template>
  <div class="graph-wrap">
    <div class="g-toolbar">
      <span class="g-title">🧩 人物关系网</span>
      <span class="g-hint">拖动节点调整布局 · 连线显示关系</span>
      <div class="g-actions">
        <button class="btn sm" @click="resetLayout">重置布局</button>
        <button class="btn sm" @click="store.fullView = null">返回写作台</button>
      </div>
    </div>
    <div class="canvas" ref="canvas" @mousemove="onMove" @mouseup="onUp" @mouseleave="onUp">
      <svg :width="CW" :height="CH" class="g-svg">
        <line
          v-for="(e, i) in edges"
          :key="i"
          :x1="e.a.x" :y1="e.a.y" :x2="e.b.x" :y2="e.b.y"
          stroke="var(--border-strong)" stroke-width="1.5"
        />
        <text
          v-for="(e, i) in edges"
          :key="'t' + i"
          :x="(e.a.x + e.b.x) / 2" :y="(e.a.y + e.b.y) / 2 - 7"
          text-anchor="middle" font-size="11" fill="var(--text-3)"
        >{{ e.relation }}</text>
      </svg>
      <div
        v-for="c in store.novel?.characters || []"
        :key="c.id"
        class="gnode"
        :class="{ sel: c.id === store.selChar }"
        :style="{ left: pos(c.id).x + 'px', top: pos(c.id).y + 'px' }"
        @mousedown="startDrag(c.id, $event)"
        @click="store.selChar = c.id"
      >
        <span class="gn-name">{{ c.name }}</span>
        <span class="gn-role" v-if="c.role">{{ c.role }}</span>
      </div>
      <div v-if="!(store.novel?.characters || []).length" class="g-empty">
        还没有人物卡，先在右栏「人物」中新建
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from "vue";
import { store } from "../store";

const CW = 2000;
const CH = 1300;
const drag = ref(null);
const localPos = ref({});

function pos(id) {
  if (!store.canvasPos) store.canvasPos = {};
  if (!store.canvasPos[id]) {
    const chars = store.novel?.characters || [];
    const i = chars.findIndex((c) => c.id === id);
    const n = Math.max(1, chars.length);
    const angle = (Math.PI * 2 * i) / n - Math.PI / 2;
    const r = Math.min(CW, CH) * 0.33;
    store.canvasPos[id] = { x: CW / 2 + r * Math.cos(angle), y: CH / 2 + r * Math.sin(angle) };
  }
  return store.canvasPos[id];
}

function startDrag(id, e) {
  e.preventDefault();
  const p = pos(id);
  drag.value = { id, dx: e.clientX - p.x - e.currentTarget.getBoundingClientRect().left + (e.currentTarget.offsetLeft || 0), dy: 0 };
}
function onMove(e) {
  if (!drag.value) return;
  const canvas = document.querySelector(".canvas");
  if (!canvas) return;
  const rect = canvas.getBoundingClientRect();
  const p = store.canvasPos[drag.value.id];
  p.x = Math.max(10, Math.min(CW - 150, e.clientX - rect.left + canvas.scrollLeft - 60));
  p.y = Math.max(10, Math.min(CH - 60, e.clientY - rect.top + canvas.scrollTop - 20));
}
function onUp() {
  drag.value = null;
}
function resetLayout() {
  store.canvasPos = {};
}

const edges = computed(() => {
  const out = [];
  for (const c of store.novel?.characters || []) {
    for (const r of c.relationships || []) {
      const b = (store.novel.characters || []).find((x) => x.id === r.target_id);
      if (b) out.push({ a: pos(c.id), b: pos(b.id), relation: r.relation });
    }
  }
  return out;
});
</script>

<style scoped>
.graph-wrap { flex: 1; display: flex; flex-direction: column; min-height: 0; }
.g-toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 16px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}
.g-title { font-size: 15px; font-weight: 600; }
.g-hint { font-size: 11.5px; color: var(--text-3); }
.g-actions { margin-left: auto; display: flex; gap: 6px; }
.canvas {
  flex: 1;
  position: relative;
  overflow: auto;
  background: var(--panel);
  margin: 12px;
  border: 1px solid var(--border);
  border-radius: 10px;
}
.g-svg { position: absolute; inset: 0; pointer-events: none; }
.gnode {
  position: absolute;
  padding: 9px 18px;
  border-radius: 10px;
  border: 1px solid var(--accent);
  background: var(--panel-alt);
  color: var(--text);
  font-size: 13px;
  cursor: grab;
  user-select: none;
  box-shadow: var(--shadow-sm);
  white-space: nowrap;
  display: flex;
  align-items: center;
  gap: 7px;
}
.gnode:hover { background: var(--hover); }
.gnode.sel { background: var(--accent-soft); border-color: var(--accent-strong); }
.gnode:active { cursor: grabbing; }
.gn-role { font-size: 10px; color: var(--text-3); }
.g-empty {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-3);
  font-size: 13px;
}
</style>
