<template>
  <button v-if="!store.aiPanelOpen" class="ai-rail" title="展开 AI 工具栏（Ctrl+J）" @click="store.aiPanelOpen = true">✨</button>
  <aside class="aipanel" :class="{ collapsed: !store.aiPanelOpen }">
    <div class="ai-head">
      <span class="ai-title">✨ AI 创作助手</span>
      <div class="ai-head-r">
        <span class="lore-label">注入设定</span>
        <label class="switch"><input type="checkbox" v-model="store.useLore" /><span class="slider"></span></label>
        <button class="icon-btn" :title="store.aiPanelOpen ? '收起工具栏' : '展开工具栏'" @click="store.aiPanelOpen = !store.aiPanelOpen">
          {{ store.aiPanelOpen ? "»" : "«" }}
        </button>
      </div>
    </div>
    <div class="sep" style="margin: 0 10px"></div>

    <!-- 预设动作 -->
    <div class="chips">
      <button v-for="c in actions" :key="c.label" class="pill sm" @click="doAction(c)">
        {{ c.label }}
      </button>
    </div>

    <!-- 对话区 -->
    <div ref="chatBox" class="chat">
      <template v-for="(m, i) in store.aiMsgs" :key="i">
        <div v-if="m.role === 'user'" class="msg user">{{ m.content }}</div>
        <div v-else-if="m.role === 'error'" class="msg err">⚠️ {{ m.content }}</div>
        <template v-else>
          <div class="msg ai">{{ m.content }}</div>
          <div class="msg-ops">
            <button class="btn sm" @click="insert(m.content)">📥 插入到正文</button>
            <button class="btn sm" @click="copy(m.content)">📋 复制</button>
            <button class="btn sm" @click="regenerate(i)">🔄 重新生成</button>
            <button v-if="m.action.includes('大纲')" class="btn sm" @click="applyOutline(m.content)">🗂️ 应用为大纲</button>
            <button v-if="['章节摘要', '摘要'].includes(m.action)" class="btn sm" @click="setSummary(m.content)">📌 设为摘要</button>
          </div>
        </template>
      </template>
      <div v-if="store.aiStreaming" class="streaming">
        <div class="stream-head"><span class="spinner"></span> {{ store.aiAction }} 生成中…</div>
        <div class="msg ai">{{ store.aiStreamText }}</div>
      </div>
    </div>

    <!-- 输入区 -->
    <div class="ai-input-row">
      <textarea
        v-model="store.aiInput"
        class="input ai-input"
        rows="2"
        placeholder="自由提问，或对 AI 结果说「再短一点」…"
        @keydown.enter.exact.prevent="sendFree"
      ></textarea>
      <button v-if="store.aiStreaming" class="icon-btn stop" title="停止生成" @click="stopAi">⏹</button>
      <button v-else class="icon-btn send" title="发送" @click="sendFree">➤</button>
    </div>
  </aside>
</template>

<script setup>
import { ref, nextTick, watch } from "vue";
import { store, toast, startAi, aiCancel, aiInsertToEditor } from "../store";
import * as prompts from "../prompts";

const chatBox = ref(null);

const actions = [
  { label: "✍️ 续写", build: () => prompts.buildContinue() },
  { label: "📝 细纲", build: () => prompts.buildChapterOutline() },
  { label: "🧹 润色", build: () => selOr(prompts.buildPolish) },
  { label: "📐 扩写", build: () => selOr(prompts.buildExpand) },
  { label: "📄 摘要", build: () => prompts.buildSummary() },
  { label: "💡 剧情", build: () => prompts.buildPlotIdeas() },
  { label: "🔍 逻辑", build: () => prompts.buildLogicCheck() },
  { label: "🧬 一致", build: () => prompts.buildConsistency() },
  { label: "📋 评审", build: () => prompts.buildFeedback() },
  { label: "👤 人物卡", build: () => prompts.buildCharacterCard("主角", "主角") },
  { label: "🌍 世界观", build: () => prompts.buildWorld() },
  { label: "🎲 起名机", local: true, run: () => (store.dialog = { kind: "namer" }) },
  { label: "📢 简介", build: () => prompts.buildSynopsis() },
];

function selOr(builder) {
  if (!store.selectedText.trim()) {
    toast("请先在正文中选中要处理的文本", false);
    return null;
  }
  return builder(store.selectedText, "");
}

function doAction(a) {
  if (a.local) {
    a.run();
    return;
  }
  const msgs = a.build();
  if (msgs) startAi(labelOf(a.label), msgs);
}
function labelOf(label) {
  const map = {
    "✍️ 续写": "续写", "📝 细纲": "章节细纲", "🧹 润色": "润色", "📐 扩写": "扩写",
    "📄 摘要": "章节摘要", "💡 剧情": "剧情提示", "🔍 逻辑": "逻辑检查", "🧬 一致": "一致性检查",
    "📋 评审": "整稿评审", "👤 人物卡": "人物卡", "🌍 世界观": "世界观", "📛 起名": "起名", "📢 简介": "简介",
  };
  return map[label] || "对话";
}

function sendFree() {
  const input = store.aiInput.trim();
  if (!input) return;
  store.aiInput = "";
  startAi("对话", prompts.buildFreeChat(input));
}

function stopAi() {
  aiCancel();
}

function insert(content) {
  aiInsertToEditor(content);
}
function copy(content) {
  navigator.clipboard.writeText(content);
  toast("已复制");
}
function regenerate(idx) {
  if (store.aiStreaming) return;
  let user = "";
  for (const m of store.aiMsgs.slice(0, idx).reverse()) {
    if (m.role === "user") { user = m.content; break; }
  }
  if (!user) return;
  store.aiMsgs = store.aiMsgs.slice(0, idx);
  startAi("对话", [user]);
}
function applyOutline(content) {
  const nodes = prompts.parseOutline(content);
  if (!nodes.length) {
    toast("未能从回复中解析出大纲结构", false);
    return;
  }
  store.novel.outline = nodes;
  apiSave();
  toast("大纲已应用（可在「大纲」面板查看）");
}
function setSummary(content) {
  const cid = store.activeTab;
  if (!cid) return;
  store.summaries[cid] = content.slice(0, 400);
  toast("已设为章节摘要（用于全稿检查）");
}
async function apiSave() {
  const { api } = await import("../api");
  api.saveNovel(store.novel);
}

// 滚动到底部
watch(
  () => [store.aiMsgs.length, store.aiStreamText.length],
  () => nextTick(() => {
    if (chatBox.value) chatBox.value.scrollTop = chatBox.value.scrollHeight;
  })
);
</script>

<style scoped>
.aipanel {
  width: 340px;
  flex-shrink: 0;
  background: var(--panel);
  border-left: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
  transition: width 0.18s ease;
}
.aipanel.collapsed {
  width: 0;
  border-left-color: transparent;
}
.aipanel.collapsed { display: none; }
.ai-rail {
  width: 36px;
  flex-shrink: 0;
  border: none;
  border-left: 1px solid var(--border);
  background: var(--panel);
  color: var(--text-2);
  font-size: 16px;
  cursor: pointer;
  transition: background 0.12s, color 0.12s;
}
.ai-rail:hover { background: var(--hover); color: var(--accent); }
.ai-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 10px 6px 14px;
}
.ai-title { font-size: 13px; font-weight: 600; }
.ai-head-r { display: flex; align-items: center; gap: 7px; }
.lore-label { font-size: 10.5px; color: var(--text-3); }
.chips {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
  padding: 8px 10px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}
.pill.sm { padding: 2px 9px; font-size: 11.5px; }
.chat { flex: 1; overflow-y: auto; padding: 10px; display: flex; flex-direction: column; gap: 8px; }
.msg {
  padding: 9px 11px;
  border-radius: 9px;
  font-size: 12.5px;
  line-height: 1.75;
  white-space: pre-wrap;
  word-break: break-word;
  max-width: 100%;
}
.msg.user { background: var(--accent-soft); color: var(--text); align-self: flex-end; }
.msg.ai { background: var(--panel-alt); color: var(--text); align-self: stretch; }
.msg.err { background: var(--danger-soft); color: var(--danger); }
.msg-ops { display: flex; gap: 5px; flex-wrap: wrap; }
.streaming { display: flex; flex-direction: column; gap: 6px; }
.stream-head { display: flex; align-items: center; gap: 8px; font-size: 11.5px; color: var(--accent); }
.spinner {
  width: 13px; height: 13px;
  border: 2px solid var(--accent-soft);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }
.ai-input-row { display: flex; align-items: flex-end; gap: 6px; padding: 8px 10px; border-top: 1px solid var(--border); }
.ai-input { flex: 1; resize: none; line-height: 1.6; }
.send { color: var(--accent); font-size: 15px; width: 30px; height: 30px; }
.send:hover { background: var(--accent-soft); }
.stop { color: var(--danger); font-size: 15px; width: 30px; height: 30px; }
</style>
