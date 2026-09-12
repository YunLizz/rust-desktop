<template>
  <div
    class="drag-handle"
    :class="{ dragging: isDragging }"
    @mousedown="start"
    @dblclick="reset"
  >
    <div class="handle-line"></div>
  </div>
</template>

<script setup>
import { ref, onBeforeUnmount } from "vue";

const props = defineProps({
  /** 当前宽度（响应式对象属性名由父组件通过 get/set 注入） */
  get: { type: Function, required: true },
  set: { type: Function, required: true },
  /** 拖拽方向：1 = 向右拖变宽（左栏），-1 = 向左拖变宽（右栏） */
  dir: { type: Number, default: 1 },
  min: { type: Number, default: 180 },
  max: { type: Number, default: 640 },
  /** 双击恢复的默认宽度 */
  fallback: { type: Number, default: 280 },
});

const isDragging = ref(false);
let startX = 0;
let startW = 0;

function start(e) {
  e.preventDefault();
  isDragging.value = true;
  startX = e.clientX;
  startW = props.get();
  document.body.style.cursor = "col-resize";
  document.body.style.userSelect = "none";
  window.addEventListener("mousemove", onMove);
  window.addEventListener("mouseup", stop);
}

function onMove(e) {
  if (!isDragging.value) return;
  const delta = (e.clientX - startX) * props.dir;
  const w = Math.max(props.min, Math.min(props.max, startW + delta));
  props.set(w);
}

function stop() {
  isDragging.value = false;
  document.body.style.cursor = "";
  document.body.style.userSelect = "";
  window.removeEventListener("mousemove", onMove);
  window.removeEventListener("mouseup", stop);
  // 持久化
  import("../store").then((m) => m.saveSettings());
}

function reset() {
  props.set(props.fallback);
  import("../store").then((m) => m.saveSettings());
}

onBeforeUnmount(stop);
</script>

<style scoped>
.drag-handle {
  width: 5px;
  flex-shrink: 0;
  cursor: col-resize;
  background: transparent;
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 5;
}
.drag-handle:hover .handle-line,
.drag-handle.dragging .handle-line {
  background: var(--accent);
}
.handle-line {
  width: 1px;
  height: 100%;
  background: var(--border);
  transition: background 0.12s, width 0.12s;
}
.drag-handle:hover .handle-line { width: 2px; }
</style>
