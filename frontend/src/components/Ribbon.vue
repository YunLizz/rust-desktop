<template>
  <div class="ribbon">
    <!-- 第一行：选项卡 -->
    <div class="ribbon-tabs" data-tauri-drag-region>
      <div class="tabs-left">
        <button
          v-for="t in tabs"
          :key="t.id"
          class="rb-tab"
          :class="{ active: store.ribbonTab === t.id }"
          @click="store.ribbonTab = t.id"
        >
          {{ t.label }}
        </button>
      </div>
      <div class="tabs-right">
        <span class="doc-title" v-if="store.novel" :title="store.novel.meta.title">
          {{ store.novel.meta.title }}<span v-if="activeChapterTitle"> · {{ activeChapterTitle }}</span>
        </span>
        <span class="dot" v-if="Object.keys(store.dirty).length" title="有未保存修改"></span>
        <button class="ctl" title="最小化" @click="minimize"><Icon name="minus" :size="14" /></button>
        <button class="ctl" :title="maximized ? '还原' : '最大化'" @click="toggleMax"><Icon :name="maximized ? 'copy' : 'square'" :size="13" /></button>
        <button class="ctl close" title="关闭" @click="close"><Icon name="close" :size="14" /></button>
      </div>
    </div>

    <!-- 第二行：当前选项卡的功能按钮组 -->
    <div class="ribbon-body">
      <template v-for="(group, gi) in currentGroups" :key="gi">
        <div class="rb-group">
          <div class="rb-btns">
            <button
              v-for="item in group.items"
              :key="item.label"
              class="rb-btn"
              :class="{ primary: item.primary }"
              :disabled="item.needNovel && !store.novel"
              :title="item.tip || item.label"
              @click="run(item)"
            >
              <span class="rb-icon">{{ item.icon }}</span>
              <span class="rb-label">{{ item.label }}</span>
            </button>
          </div>
          <div class="rb-group-name">{{ group.name }}</div>
        </div>
      </template>
      <button
        class="rb-collapse"
        :title="store.timelineOpen ? '收起时间轴' : '展开时间轴'"
        @click="store.timelineOpen = !store.timelineOpen"
      >{{ store.timelineOpen ? "▴" : "▾" }}</button>
    </div>

  </div>
</template>

<script setup>
import { ref, computed, onMounted } from "vue";
import { store, allChapters, saveAll, toast, openTab } from "../store";
import { api } from "../api";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Icon from "./Icon.vue";

const tabs = [
  { id: "writing", label: "写作" },
  { id: "lore", label: "设定" },
  { id: "tools", label: "工具" },
  { id: "view", label: "视图" },
];

const activeChapterTitle = computed(() => {
  const c = allChapters(store.novel).find((x) => x.id === store.activeTab);
  return c?.title || "";
});

// 依据当前选项卡生成按钮组
const currentGroups = computed(() => {
  const groups = [];
  if (store.ribbonTab === "writing") {
    groups.push({
      name: "章节",
      items: [
        { icon: "＋", label: "新建", run: () => (store.dialog = { kind: "newChapter" }), needNovel: true },
        { icon: "📁", label: "新建卷", run: () => (store.dialog = { kind: "newVolume" }), needNovel: true },
        { icon: "✏️", label: "重命名", run: renameChapter, needNovel: true },
        { icon: "🗑", label: "删除", run: deleteChapter, needNovel: true },
      ],
    });
    groups.push({
      name: "排版",
      items: [
        { icon: "␣", label: "首行缩进", run: () => emitEditorCmd("indent"), needNovel: true },
        { icon: "↔", label: "两端对齐", run: toggleJustify, needNovel: true },
        { icon: "A+", label: "增大字号", run: () => bumpFont(1) },
        { icon: "A−", label: "减小字号", run: () => bumpFont(-1) },
      ],
    });
    groups.push({
      name: "文件",
      items: [
        { icon: "💾", label: "保存", run: () => saveAll().then(() => toast("已保存（加密写入本地）")), primary: true },
        { icon: "📤", label: "导出", run: () => (store.dialog = { kind: "export" }), needNovel: true },
      ],
    });
  } else if (store.ribbonTab === "lore") {
    groups.push({
      name: "人物",
      items: [
        { icon: "＋", label: "新建人物", run: newCharacter, needNovel: true },
        { icon: "🧩", label: "关系网", run: () => (store.fullView = "graph"), needNovel: true },
      ],
    });
    groups.push({
      name: "世界观",
      items: [
        { icon: "＋", label: "新建地点", run: newLocation, needNovel: true },
        { icon: "⏱️", label: "新建事件", run: newEvent, needNovel: true },
      ],
    });
    groups.push({
      name: "大纲",
      items: [
        { icon: "📋", label: "从章节生成", run: outlineFromChapters, needNovel: true },
        { icon: "＋", label: "新建节点", run: addOutlineNode, needNovel: true },
      ],
    });
    groups.push({
      name: "任务",
      items: [
        { icon: "＋", label: "新建任务", run: newTask, needNovel: true },
        { icon: "🔗", label: "新建任务链", run: newChain, needNovel: true },
      ],
    });
  } else if (store.ribbonTab === "tools") {
    groups.push({
      name: "查找",
      items: [
        { icon: "🔍", label: "查找替换", run: () => emitEditorCmd("find"), needNovel: true },
        { icon: "🔎", label: "全书搜索", run: () => { store.rightTool = "namer"; store.rightOpen = true; } },
      ],
    });
    groups.push({
      name: "起名",
      items: [
        { icon: "🎲", label: "起名机", run: () => (store.dialog = { kind: "namer" }), primary: true },
      ],
    });
    groups.push({
      name: "数据",
      items: [
        { icon: "📚", label: "书库", run: () => (store.fullView = "library") },
        { icon: "📊", label: "统计", run: () => (store.fullView = "stats"), needNovel: true },
        { icon: "📂", label: "数据目录", run: () => api.openDir(store.dataDir) },
        { icon: "⚙️", label: "设置", run: () => (store.fullView = "settings") },
      ],
    });
  } else {
    groups.push({
      name: "左栏（树形结构）",
      items: [
        { icon: "📑", label: "章节树", run: () => setLeftTree("chapters") },
        { icon: "🗂️", label: "大纲树", run: () => setLeftTree("outline") },
        { icon: "👥", label: "人物树", run: () => setLeftTree("characters") },
        { icon: "🗺️", label: "世界树", run: () => setLeftTree("world") },
        { icon: "⊞", label: store.leftOpen ? "隐藏左栏" : "显示左栏", run: () => (store.leftOpen = !store.leftOpen) },
      ],
    });
    groups.push({
      name: "顶栏横向轴",
      items: [
        { icon: "🎬", label: store.timelineOpen ? "收起时间轴" : "展开时间轴", run: () => (store.timelineOpen = !store.timelineOpen), primary: store.timelineOpen },
      ],
    });
    groups.push({
      name: "右栏（创意工具）",
      items: [
        { icon: "🧰", label: "工具格", run: () => openTool(null) },
        { icon: "🎲", label: "起名机", run: () => openTool("namer") },
        { icon: "🗺", label: "地图", run: () => openTool("map") },
        { icon: "🧩", label: "关系网", run: () => openTool("graph") },
        { icon: "⏱", label: "时间轴", run: () => openTool("timeline") },
        { icon: "🎯", label: "任务", run: () => openTool("tasks") },
        { icon: "⊞", label: store.rightOpen ? "隐藏右栏" : "显示右栏", run: () => (store.rightOpen = !store.rightOpen) },
      ],
    });
    groups.push({
      name: "窗口",
      items: [
        { icon: "🎯", label: store.focusMode ? "退出专注" : "专注写作", run: () => (store.focusMode = !store.focusMode), primary: true },
        { icon: "🌗", label: "深浅主题", run: toggleTheme },
        { icon: "⚙️", label: "正文设置", run: () => (store.fullView = "settings") },
      ],
    });
  }
  return groups;
});

// ---- 动作 ----
function run(item) {
  item.run?.();
}
function emitEditorCmd(cmd) {
  window.dispatchEvent(new CustomEvent("jinshu:editor-cmd", { detail: { cmd } }));
}
function setLeftTree(p) {
  store.leftTree = p;
  store.leftOpen = true;
  store.fullView = null;
}
function openTool(t) {
  store.rightTool = t;
  store.rightOpen = true;
  store.fullView = null;
}
function bumpFont(d) {
  const e = store.settings.editor;
  e.font_size = Math.max(12, Math.min(32, (e.font_size || 17) + d));
  import("../store").then((m) => {
    m.saveSettings();
    emitEditorCmd("refresh-theme");
  });
}
function toggleJustify() {
  store.settings.editor.justify = !store.settings.editor.justify;
  import("../store").then((m) => m.saveSettings());
  emitEditorCmd("refresh-theme");
  toast(store.settings.editor.justify ? "已开启两端对齐" : "已关闭两端对齐");
}
function toggleTheme() {
  store.settings.theme = store.settings.theme === "light" ? "dark" : "light";
  import("../store").then((m) => {
    m.saveSettings();
    m.applyTheme();
  });
}
function renameChapter() {
  const c = allChapters(store.novel).find((x) => x.id === store.activeTab);
  if (!c) return toast("请先打开一个章节", false);
  store.dialog = { kind: "renameChapter", payload: { cid: c.id } };
}
function deleteChapter() {
  const c = allChapters(store.novel).find((x) => x.id === store.activeTab);
  if (!c) return toast("请先打开一个章节", false);
  store.dialog = { kind: "deleteChapter", payload: { cid: c.id, title: c.title } };
}
function newCharacter() {
  const c = { id: "c" + Math.random().toString(36).slice(2, 8), name: "新人物", role: "", appearance: "", personality: "", background: "", goals: "", notes: "", relationships: [] };
  store.novel.characters.push(c);
  store.selChar = c.id;
  store.rightTool = "graph";
  store.rightOpen = true;
  api.saveNovel(store.novel);
}
function newLocation() {
  const l = { id: "l" + Math.random().toString(36).slice(2, 8), name: "新地点", kind: "城市", parent_id: null, description: "" };
  store.novel.locations.push(l);
  api.saveNovel(store.novel);
  toast("已新建地点（在「设定」选项卡查看）");
}
function newEvent() {
  const e = { id: "e" + Math.random().toString(36).slice(2, 8), title: "新事件", time: "第1卷", description: "", character_ids: [], location_id: null, chapter_id: store.activeTab };
  store.novel.timeline.push(e);
  store.rightTool = "timeline";
  store.rightOpen = true;
  api.saveNovel(store.novel);
}
function newTask() {
  const t = { id: "t" + Math.random().toString(36).slice(2, 8), title: "新任务", description: "", status: 0, chain_id: null };
  store.novel.tasks.push(t);
  store.rightTool = "tasks";
  store.rightOpen = true;
  api.saveNovel(store.novel);
}
function newChain() {
  const ch = { id: "h" + Math.random().toString(36).slice(2, 8), name: "新任务链", description: "", task_ids: [] };
  store.novel.chains.push(ch);
  api.saveNovel(store.novel);
  toast("已新建任务链");
}
function addOutlineNode() {
  store.novel.outline.push({
    id: "o" + Math.random().toString(36).slice(2, 8),
    title: "新节点",
    kind: "卷",
    content: "",
    children: [],
  });
  store.leftTree = "outline";
  store.leftOpen = true;
  api.saveNovel(store.novel);
}
function outlineFromChapters() {
  const n = store.novel;
  if (!n?.volumes?.length) return toast("暂无章节", false);
  n.outline = n.volumes.map((v) => ({
    id: "o" + Math.random().toString(36).slice(2, 8),
    title: v.title,
    kind: "卷",
    content: "",
    children: (v.chapters || []).map((c) => ({
      id: "o" + Math.random().toString(36).slice(2, 8),
      title: c.title,
      kind: "章",
      content: "",
      children: [],
    })),
  }));
  api.saveNovel(n);
  store.leftTree = "outline";
  store.leftOpen = true;
  toast("已从章节生成大纲骨架");
}

// ---- 横向时间轴 ----

// ---- 窗口控制 ----
const maximized = ref(false);
let appWindow = null;
async function initWindow() {
  try {
    appWindow = getCurrentWindow();
    maximized.value = await appWindow.isMaximized();
    appWindow.onResized(() => appWindow.isMaximized().then((m) => (maximized.value = m)));
  } catch (e) {}
}
const minimize = () => appWindow?.minimize();
const toggleMax = () => appWindow?.toggleMaximize();
const close = () => appWindow?.close();
onMounted(initWindow);
</script>

<style scoped>
.ribbon {
  flex-shrink: 0;
  background: var(--chrome);
  border-bottom: 1px solid var(--border);
  user-select: none;
}
/* --- 第一行：选项卡 --- */
.ribbon-tabs {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 34px;
  padding-left: 6px;
}
.tabs-left { display: flex; gap: 2px; height: 100%; align-items: stretch; }
.rb-tab {
  border: none;
  background: transparent;
  color: var(--text-2);
  font-size: 13px;
  padding: 0 16px;
  cursor: pointer;
  font-family: inherit;
  border-bottom: 2px solid transparent;
  transition: color 0.12s, background 0.12s;
}
.rb-tab:hover { color: var(--text); background: var(--hover); }
.rb-tab.active { color: var(--accent); border-bottom-color: var(--accent); font-weight: 500; }
.tabs-right { display: flex; align-items: center; gap: 4px; height: 100%; }
.doc-title {
  font-size: 11.5px;
  color: var(--text-3);
  max-width: 320px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin-right: 6px;
}
.dot { width: 6px; height: 6px; border-radius: 50%; background: var(--warn); margin-right: 4px; }
.ctl {
  width: 42px;
  height: 100%;
  border: none;
  background: transparent;
  color: var(--text-2);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.12s, color 0.12s;
}
.ctl:hover { background: var(--hover); color: var(--text); }
.ctl.close:hover { background: var(--danger); color: #fff; }

/* --- 第二行：功能按钮 --- */
.ribbon-body {
  display: flex;
  align-items: stretch;
  gap: 0;
  height: 58px;
  padding: 4px 8px 0;
  border-top: 1px solid var(--border);
  overflow-x: auto;
  overflow-y: hidden;
}
.rb-group {
  display: flex;
  flex-direction: column;
  padding: 0 10px;
  border-right: 1px solid var(--border);
  flex-shrink: 0;
}
.rb-group:last-child { border-right: none; }
.rb-btns {
  display: flex;
  gap: 3px;
  flex: 1;
  align-items: center;
  flex-wrap: wrap;
  max-height: 42px;
  align-content: center;
}
.rb-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
  padding: 5px 9px;
  border: 1px solid transparent;
  border-radius: 6px;
  background: transparent;
  color: var(--text);
  cursor: pointer;
  font-family: inherit;
  transition: background 0.12s, border-color 0.12s;
}
.rb-btn:hover:not(:disabled) { background: var(--hover); border-color: var(--border); }
.rb-btn.primary { background: var(--accent-soft); }
.rb-btn.primary:hover:not(:disabled) { background: var(--accent); color: #fff; }
.rb-btn:disabled { opacity: 0.35; cursor: not-allowed; }
.rb-icon { font-size: 14px; line-height: 1; }
.rb-label { font-size: 12px; white-space: nowrap; }
.rb-collapse {
  margin-left: auto;
  align-self: flex-start;
  border: none;
  background: transparent;
  color: var(--text-3);
  font-size: 11px;
  cursor: pointer;
  padding: 2px 8px;
  border-radius: 5px;
  flex-shrink: 0;
}
.rb-collapse:hover { background: var(--hover); color: var(--text); }

/* 横向时间轴 */
.tl-strip {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 12px;
  border-top: 1px solid var(--border);
  background: var(--panel);
  flex-shrink: 0;
  height: 52px;
}
.tls-label {
  font-size: 10.5px;
  color: var(--text-3);
  flex-shrink: 0;
  letter-spacing: 0.03em;
}
.tls-track {
  position: relative;
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: flex-start;
  gap: 0;
  overflow-x: auto;
  overflow-y: hidden;
  height: 100%;
  padding: 0 12px;
}
.tls-line {
  position: absolute;
  left: 12px;
  right: 12px;
  top: 50%;
  height: 1px;
  background: var(--border-strong);
}
.tls-node {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1px;
  padding: 0 16px;
  cursor: pointer;
  flex-shrink: 0;
  z-index: 1;
  min-width: 96px;
}
.tls-dot {
  width: 9px;
  height: 9px;
  border-radius: 50%;
  background: var(--panel);
  border: 2px solid var(--border-strong);
  transition: all 0.13s;
}
.tls-node:hover .tls-dot { border-color: var(--accent); transform: scale(1.2); }
.tls-node.linked .tls-dot { border-color: var(--accent); }
.tls-node.cur .tls-dot { background: var(--accent); border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent-soft); }
.tls-time { font-size: 9px; color: var(--text-3); margin-top: 2px; }
.tls-name { font-size: 10.5px; color: var(--text-2); max-width: 90px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.tls-node.cur .tls-name { color: var(--accent); font-weight: 500; }
.tls-empty { font-size: 11px; color: var(--text-3); padding-left: 8px; z-index: 1; }
.tls-add {
  border: 1px dashed var(--border);
  background: transparent;
  color: var(--text-3);
  border-radius: 6px;
  width: 28px;
  height: 28px;
  cursor: pointer;
  flex-shrink: 0;
  font-size: 13px;
}
.tls-add:hover { border-color: var(--accent); color: var(--accent); }

.rb-group-name {
  text-align: center;
  font-size: 9.5px;
  color: var(--text-3);
  padding: 1px 0 2px;
}
</style>
