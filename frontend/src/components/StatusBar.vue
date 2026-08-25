<template>
  <footer class="statusbar">
    <div class="left">
      <span class="item ok">🔒 加密存储</span>
      <template v-if="store.novel">
        <span class="item">{{ store.novel.meta.title }}</span>
        <span class="item">字数 {{ totalWords }}</span>
        <span class="item warn">今日 +{{ todayWords }}</span>
        <span class="item dim" v-if="store.cursorPos">行 {{ store.cursorPos.line }}，列 {{ store.cursorPos.col }}</span>
      </template>
    </div>
    <div class="right">
      <span class="item dim" :class="{ streaming: store.aiStreaming }">{{ store.aiStreaming ? "✨ AI 生成中…" : "✨ AI 就绪" }}</span>
      <span class="item dim">UTF-8</span>
      <span class="item dim" :title="store.dataDir">🔑 {{ shortFp }}</span>
    </div>
  </footer>
</template>

<script setup>
import { computed } from "vue";
import { store, today } from "../store";

const totalWords = computed(() => store.novel?.meta?.total_words || 0);
const todayWords = computed(() => store.novel?.stats?.[today()] || 0);
const shortFp = computed(() => {
  const fp = store.keyFp || "";
  return fp.length > 8 ? fp.slice(0, 4) + "…" + fp.slice(-4) : fp;
});
</script>

<style scoped>
.statusbar {
  height: 26px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--chrome);
  border-top: 1px solid var(--border);
  font-size: 11px;
  padding: 0 12px;
  user-select: none;
}
.left, .right { display: flex; align-items: center; gap: 14px; min-width: 0; }
.item { color: var(--text-2); white-space: nowrap; }
.item.ok { color: var(--ok); }
.item.warn { color: var(--warn); }
.item.dim { color: var(--text-3); }
.streaming { color: var(--accent); }
</style>
