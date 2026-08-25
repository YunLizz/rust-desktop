<template>
  <nav class="activity" :class="{ expanded }">
    <div class="act-list">
      <button
        v-for="a in items"
        :key="a.id"
        class="act"
        :class="{ active: store.activity === a.id }"
        :title="a.label"
        @click="select(a.id)"
      >
        <span class="bar" v-if="store.activity === a.id"></span>
        <Icon :name="a.icon" :size="19" />
        <span class="act-label" v-if="expanded">{{ a.label }}</span>
      </button>
    </div>
    <button class="act-toggle" :title="expanded ? '收起导航栏' : '展开导航栏'" @click="toggle">
      <span class="toggle-arrow">{{ expanded ? "«" : "»" }}</span>
    </button>
  </nav>
</template>

<script setup>
import { computed } from "vue";
import { store } from "../store";
import { api } from "../api";
import Icon from "./Icon.vue";

const expanded = computed({
  get: () => store.settings?.nav_expanded !== false,
  set: (v) => {
    store.settings.nav_expanded = v;
    import("../store").then((m) => m.saveSettings());
  },
});
function toggle() {
  expanded.value = !expanded.value;
}

const items = [
  { id: "library", icon: "library", label: "书库" },
  { id: "chapters", icon: "chapters", label: "章节" },
  { id: "outline", icon: "outline", label: "大纲" },
  { id: "characters", icon: "characters", label: "人物" },
  { id: "world", icon: "world", label: "世界观" },
  { id: "timeline", icon: "timeline", label: "时间线" },
  { id: "tasks", icon: "tasks", label: "任务" },
  { id: "search", icon: "search", label: "搜索" },
  { id: "stats", icon: "stats", label: "统计" },
  { id: "settings", icon: "settings", label: "设置" },
];

async function select(id) {
  store.activity = id;
  if (id === "library" || id === "stats" || id === "settings") {
    store.sidebarOpen = false;
    if (id === "library") store.library = await api.listNovels();
  } else {
    store.sidebarOpen = true;
  }
}
</script>

<style scoped>
.activity {
  width: 48px;
  flex-shrink: 0;
  background: var(--chrome);
  display: flex;
  flex-direction: column;
  padding-top: 8px;
  border-right: 1px solid var(--border);
  transition: width 0.15s ease;
  overflow: hidden;
}
.activity.expanded { width: 172px; }
.act-list { display: flex; flex-direction: column; align-items: center; gap: 2px; }
.activity.expanded .act-list { align-items: stretch; padding: 0 6px; }
.act {
  position: relative;
  width: 40px;
  height: 40px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  margin: 0 auto;
  border: none;
  background: transparent;
  border-radius: 9px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 17px;
  color: var(--text-3);
  transition: background 0.12s, color 0.12s;
}
.act:hover { background: var(--hover); color: var(--text); }
.act.active { background: var(--accent-soft); color: var(--accent); }
.activity.expanded .act { width: 100%; justify-content: flex-start; padding-left: 12px; }
.act-label { font-size: 12.5px; color: inherit; white-space: nowrap; }
.act-toggle {
  margin-top: auto;
  margin-bottom: 10px;
  align-self: center;
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: var(--text-3);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.12s, color 0.12s;
}
.act-toggle:hover { background: var(--hover); color: var(--text); }
.toggle-arrow { font-size: 15px; line-height: 1; }
.activity.expanded .act-toggle { align-self: flex-end; margin-right: 8px; }
.act .bar {
  position: absolute;
  left: -7px;
  width: 3px;
  height: 20px;
  border-radius: 2px;
  background: var(--accent);
}
</style>
