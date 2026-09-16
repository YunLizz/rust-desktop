<template>
  <footer class="statusbar">
    <div class="left">
      <span class="item ok">🔒 加密存储</span>
      <span class="sep" v-if="store.novel"></span>
      <template v-if="store.novel">
        <span class="item book" :title="store.novel.meta.title">{{ store.novel.meta.title }}</span>
        <span class="item">总字数 <b>{{ totalWords.toLocaleString() }}</b></span>
        <span class="item warn">今日 <b>+{{ todayWords.toLocaleString() }}</b></span>
        <span class="item" v-if="chapterWords !== null && chapterWords !== totalWords">本章 <b>{{ chapterWords.toLocaleString() }}</b></span>
        <span class="item dim" v-if="store.cursorPos">行 {{ store.cursorPos.line }} · 列 {{ store.cursorPos.col }}</span>
      </template>
    </div>
    <div class="right">
      <span class="item dim">UTF-8</span>
      <span class="item dim" :title="'本地加密密钥指纹（数据目录：' + store.dataDir + '）'">🔑 密钥 {{ shortFp }}</span>
      <button class="item link" title="打开数据目录" @click="openDir">📂</button>
    </div>
  </footer>
</template>

<script setup>
import { computed } from "vue";
import { store, today, allChapters } from "../store";
import { api } from "../api";

const totalWords = computed(() => store.novel?.meta?.total_words || 0);
const todayWords = computed(() => store.novel?.stats?.[today()] || 0);
const chapterWords = computed(() => {
  const c = allChapters(store.novel).find((x) => x.id === store.activeTab);
  return c ? c.words || 0 : null;
});
const shortFp = computed(() => {
  const fp = store.keyFp || "";
  return fp.length > 8 ? fp.slice(0, 4) + "…" + fp.slice(-4) : fp || "—";
});
function openDir() {
  api.openDir(store.dataDir);
}
</script>

<style scoped>
/* 加高到 34px、字号 13px，字数数字加重，便于扫读 */
.statusbar {
  height: 34px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--chrome);
  border-top: 1px solid var(--border);
  font-size: 13px;
  padding: 0 14px;
  user-select: none;
}
.left,
.right {
  display: flex;
  align-items: center;
  gap: 16px;
  min-width: 0;
}
.item {
  color: var(--text-2);
  white-space: nowrap;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}
.item b {
  color: var(--text);
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}
.item.ok { color: var(--ok); }
.item.warn { color: var(--warn); }
.item.dim { color: var(--text-3); font-size: 12.5px; }
.item.book {
  max-width: 280px;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--text);
  font-weight: 500;
}
.sep {
  width: 1px;
  height: 15px;
  background: var(--border);
  flex-shrink: 0;
}
.link {
  border: none;
  background: transparent;
  cursor: pointer;
  font-size: 14px;
  padding: 2px 6px;
  border-radius: 5px;
  color: var(--text-3);
}
.link:hover { background: var(--hover); color: var(--text); }
</style>
