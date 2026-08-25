<template>
  <div class="stats" v-if="store.novel">
    <div class="st-head">
      <span class="st-title">📊 写作统计</span>
      <span class="st-novel">《{{ store.novel.meta.title }}》</span>
    </div>

    <div class="cards-row">
      <div class="st-card" v-for="c in cards" :key="c.label">
        <div class="stc-value" :style="{ color: c.color }">{{ c.value }}</div>
        <div class="stc-label">{{ c.label }}</div>
      </div>
    </div>

    <div class="chart-block">
      <div class="cb-title">近 30 天字数</div>
      <div class="chart">
        <div
          v-for="(e, i) in chartData"
          :key="i"
          class="bar-col"
          :title="e.date + '：' + e.v + ' 字'"
        >
          <div class="bar" :style="{ height: barH(e.v) + 'px' }" :class="{ zero: !e.v }"></div>
          <div class="bar-label" v-if="i % 5 === 0">{{ e.date.slice(5) }}</div>
        </div>
      </div>
    </div>

    <div class="vol-block">
      <span class="cb-title">各卷字数</span>
      <div class="vol-list">
        <span v-for="v in store.novel.volumes" :key="v.id" class="vol-item">
          {{ v.title }}：{{ volWords(v) }} 字
        </span>
      </div>
    </div>
  </div>
  <div v-else class="empty">
    <div class="emoji">📊</div>
    <div class="title">请先打开一部小说</div>
  </div>
</template>

<script setup>
import { computed } from "vue";
import { store, today , allChapters } from "../store";

const cards = computed(() => {
  const n = store.novel;
  const stats = n.stats || {};
  const t = today();
  const total = Object.values(stats).reduce((a, b) => a + b, 0);
  return [
    { label: "总字数", value: String(n.meta.total_words || 0), color: "var(--text)" },
    { label: "今日新增", value: "+" + (stats[t] || 0), color: "var(--warn)" },
    { label: "累计写作", value: String(total), color: "var(--accent)" },
    { label: "连续创作(天)", value: String(streak()), color: "var(--ok)" },
    { label: "创作天数", value: String(Object.keys(stats).length), color: "var(--purple)" },
    { label: "章节数", value: String(allChapters(store.novel).length), color: "var(--cyan)" },
  ];
});

function daysAgo(n) {
  const d = new Date();
  d.setDate(d.getDate() - n);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}

const chartData = computed(() => {
  const stats = store.novel.stats || {};
  const out = [];
  for (let i = 29; i >= 0; i--) {
    const d = daysAgo(i);
    out.push({ date: d, v: stats[d] || 0 });
  }
  return out;
});
const maxV = computed(() => Math.max(1, ...chartData.value.map((e) => e.v)));
function barH(v) {
  return Math.max(2, (v / maxV.value) * 150);
}

function streak() {
  const stats = store.novel.stats || {};
  let day = new Date();
  if (!stats[daysAgo(0)]) day.setDate(day.getDate() - 1);
  let s = 0;
  while (true) {
    const key = `${day.getFullYear()}-${String(day.getMonth() + 1).padStart(2, "0")}-${String(day.getDate()).padStart(2, "0")}`;
    if (!stats[key]) break;
    s++;
    day.setDate(day.getDate() - 1);
    if (s > 3000) break;
  }
  return s;
}

function volWords(v) {
  return v.chapters.reduce((a, c) => a + (c.words || 0), 0);
}
</script>

<style scoped>
.stats { flex: 1; overflow-y: auto; padding: 18px 22px; }
.st-head { display: flex; align-items: baseline; gap: 12px; margin-bottom: 16px; }
.st-title { font-size: 19px; font-weight: 700; }
.st-novel { font-size: 13px; color: var(--text-2); }
.cards-row { display: flex; gap: 14px; flex-wrap: wrap; margin-bottom: 22px; }
.st-card {
  width: 152px;
  height: 78px;
  border-radius: 10px;
  border: 1px solid var(--border);
  background: var(--panel);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
}
.stc-value { font-size: 20px; font-weight: 700; }
.stc-label { font-size: 11.5px; color: var(--text-2); }
.chart-block { margin-bottom: 20px; }
.cb-title { font-size: 14px; font-weight: 600; margin-bottom: 10px; }
.chart {
  display: flex;
  align-items: flex-end;
  gap: 5px;
  height: 175px;
  padding: 0 6px;
  border-bottom: 1px solid var(--border);
  overflow-x: auto;
}
.bar-col { display: flex; flex-direction: column; align-items: center; justify-content: flex-end; gap: 4px; }
.bar {
  width: 20px;
  border-radius: 3px 3px 0 0;
  background: var(--accent);
  transition: height 0.2s;
}
.bar.zero { background: var(--panel-alt); }
.bar-label { font-size: 9.5px; color: var(--text-3); }
.vol-block { display: flex; align-items: baseline; gap: 14px; }
.vol-list { display: flex; gap: 14px; flex-wrap: wrap; font-size: 12px; color: var(--text-2); }
</style>
