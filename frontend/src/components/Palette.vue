<template>
  <div class="overlay" @mousedown.self="store.paletteOpen = false">
    <div class="popup palette slide-up" @keydown.stop>
      <input
        ref="input"
        v-model="store.paletteQuery"
        class="palette-input"
        placeholder="输入命令…（Esc 关闭）"
        autofocus
        @keydown.enter="exec(first)"
        @keydown.up.prevent="move(-1)"
        @keydown.down.prevent="move(1)"
      />
      <div class="list">
        <button
          v-for="(item, i) in filtered"
          :key="item.label"
          class="palette-item"
          :class="{ sel: i === sel }"
          @mouseenter="sel = i"
          @click="exec(item)"
        >
          <span class="pi-icon">{{ item.icon }}</span>
          <span class="pi-label">{{ item.label }}</span>
          <span class="pi-kbd">{{ item.kbd }}</span>
        </button>
        <div v-if="!filtered.length" class="none">没有匹配的命令</div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch } from "vue";
import { store, toast } from "../store";
import { api } from "../api";
import * as prompts from "../prompts";

const sel = ref(0);
const input = ref(null);

const commands = [
  { icon: "📝", label: "新建小说", kbd: "Ctrl+N", run: () => (store.dialog = { kind: "newNovel" }) },
  { icon: "📚", label: "打开书库", kbd: "Ctrl+O", run: () => { store.activity = "library"; store.library = api.listNovels(); } },
  { icon: "💾", label: "保存全部", kbd: "Ctrl+S", run: async () => { const { saveAll } = await import("../store"); await saveAll(); toast("已保存（加密写入本地）"); } },
  { icon: "📤", label: "导出作品", run: () => store.novel ? (store.dialog = { kind: "export" }) : toast("请先打开一部小说", false) },
  { icon: "📥", label: "导入 .jsb 备份", run: () => (store.dialog = { kind: "import" }) },
  { icon: "📄", label: "新建章节", run: () => store.novel ? (store.dialog = { kind: "newChapter" }) : null },
  { icon: "📁", label: "新建分卷", run: () => store.novel ? (store.dialog = { kind: "newVolume" }) : null },
  { icon: "✍️", label: "AI 续写", run: () => startAi("续写", prompts.buildContinue()) },
  { icon: "🗂️", label: "AI 生成大纲", run: () => startAi("大纲生成", prompts.buildOutline()) },
  { icon: "📝", label: "AI 生成细纲", run: () => startAi("章节细纲", prompts.buildChapterOutline()) },
  { icon: "🧹", label: "AI 润色选中文本", run: () => selText("润色", prompts.buildPolish) },
  { icon: "📐", label: "AI 扩写选中文本", run: () => selText("扩写", prompts.buildExpand) },
  { icon: "📄", label: "AI 章节摘要", run: () => startAi("章节摘要", prompts.buildSummary()) },
  { icon: "💡", label: "AI 剧情提示", run: () => startAi("剧情提示", prompts.buildPlotIdeas()) },
  { icon: "🔍", label: "AI 逻辑检查", run: () => startAi("逻辑检查", prompts.buildLogicCheck()) },
  { icon: "🧬", label: "AI 一致性检查", run: () => startAi("一致性检查", prompts.buildConsistency()) },
  { icon: "📋", label: "AI 整稿评审", run: () => startAi("整稿评审", prompts.buildFeedback()) },
  { icon: "👤", label: "AI 人物卡", run: () => startAi("人物卡", prompts.buildCharacterCard("主角", "主角")) },
  { icon: "🌍", label: "AI 世界观", run: () => startAi("世界观", prompts.buildWorld()) },
  { icon: "🎲", label: "本地起名机（人物/书名/地名）", run: () => (store.dialog = { kind: "namer" }) },
  { icon: "📢", label: "AI 简介", run: () => startAi("简介", prompts.buildSynopsis()) },
  { icon: "🖥️", label: "切换侧边栏", kbd: "Ctrl+B", run: () => (store.sidebarOpen = !store.sidebarOpen) },
  { icon: "✨", label: "切换 AI 面板", kbd: "Ctrl+J", run: () => (store.aiPanelOpen = !store.aiPanelOpen) },
  { icon: "🌗", label: "切换深色/浅色主题", run: async () => {
      const { applyTheme, saveSettings } = await import("../store");
      store.settings.theme = store.settings.theme === "dark" ? "light" : "dark";
      applyTheme(); saveSettings();
    } },
  { icon: "🔍", label: "查找", kbd: "Ctrl+F", run: () => (store.findOpen = true) },
  { icon: "📊", label: "写作统计", run: () => (store.activity = "stats") },
  { icon: "⚙️", label: "打开设置", run: () => (store.activity = "settings") },
  { icon: "ℹ️", label: "关于锦书", run: () => (store.dialog = { kind: "about" }) },
];

function startAi(action, msgs) {
  const { startAi } = window.__jinshu ?? {};
  import("../store").then((m) => m.startAi(action, msgs));
  store.aiPanelOpen = true;
  store.activity = "chapters";
}
function selText(action, builder) {
  if (!store.selectedText.trim()) { toast("请先在正文中选中要处理的文本", false); return; }
  startAi(action, builder(store.selectedText, ""));
}

const filtered = computed(() => {
  const q = store.paletteQuery.toLowerCase().trim();
  if (!q) return commands;
  return commands.filter((c) => c.label.toLowerCase().includes(q));
});
watch(filtered, () => (sel.value = 0));

function move(d) {
  const n = filtered.value.length;
  sel.value = (sel.value + d + n) % n;
}
function exec(item) {
  if (!item) return;
  store.paletteOpen = false;
  item.run();
}
onMounted(() => setTimeout(() => input.value?.focus(), 30));
</script>

<style scoped>
.palette { width: 560px; max-width: 90vw; overflow: hidden; }
.palette-input {
  width: 100%;
  border: none;
  outline: none;
  background: var(--panel);
  color: var(--text);
  font-size: 13.5px;
  padding: 12px 14px;
  border-radius: 8px;
  font-family: inherit;
}
.list { max-height: 380px; overflow-y: auto; padding: 6px; }
.palette-item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 7px 10px;
  border: none;
  background: transparent;
  border-radius: 6px;
  cursor: pointer;
  font-family: inherit;
  font-size: 13px;
  color: var(--text);
  text-align: left;
}
.palette-item.sel { background: var(--accent-soft); }
.palette-item .pi-icon { width: 20px; text-align: center; }
.palette-item .pi-kbd { margin-left: auto; color: var(--text-3); font-size: 11px; }
.none { padding: 16px; text-align: center; color: var(--text-3); font-size: 12.5px; }
</style>
