<template>
  <div class="editor-wrap">
    <!-- 标签条 -->
    <div class="tabbar" v-if="!store.focusMode">
      <div
        v-for="cid in store.openTabs"
        :key="cid"
        class="tab"
        :class="{ active: cid === store.activeTab }"
        @click="activate(cid)"
        @contextmenu.prevent="openTabMenu($event, cid)"
      >
        <span class="tab-title">{{ shortTitle(cid) }}</span>
        <span class="dot" v-if="store.dirty[cid]"></span>
        <span class="tab-close" @click.stop="closeTabLocal(cid)">✕</span>
      </div>
      <div class="tabbar-grow"></div>
    </div>

    <!-- 章节头 -->
    <div class="chapter-head">
      <input
        class="ch-title"
        v-model="chTitle"
        placeholder="章节标题"
        @change="saveTitle"
      />
      <span class="ch-words">{{ store.wordCount }} 字</span>
      <div class="ch-actions">
        <button class="btn sm" @click="quickContinue">✨ 续写</button>
        <button class="btn sm" @click="saveNow">💾 保存</button>
      </div>
    </div>

    <!-- 查找替换条 -->
    <div class="findbar" v-if="store.findOpen">
      <input class="input find-input" v-model="findQ" placeholder="查找…" @keydown.enter.prevent="findNext" @input="updateFind" />
      <span class="find-count" :class="{ none: !findTotal }">{{ findTotal ? (findCur + 1) + "/" + findTotal : "无结果" }}</span>
      <button class="btn sm" @click="findPrev">上一个</button>
      <button class="btn sm" @click="findNext">下一个</button>
      <button class="btn sm" @click="store.findReplace = !store.findReplace">替换</button>
      <template v-if="store.findReplace">
        <input class="input find-input" v-model="findR" placeholder="替换为…" />
        <button class="btn sm" @click="replaceCur">替换当前</button>
        <button class="btn sm" @click="replaceAll">全部替换</button>
      </template>
      <button class="icon-btn" @click="store.findOpen = false">✕</button>
    </div>

    <!-- CodeMirror -->
    <div ref="cmHost" class="cm-host"></div>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted, onBeforeUnmount, nextTick } from "vue";
import { store, markDirty, saveChapterNow, saveAll, countWords, toast , allChapters } from "../store";
import { api } from "../api";
import * as prompts from "../prompts";

import { EditorView, keymap, lineNumbers, highlightActiveLine, drawSelection, highlightSpecialChars, placeholder } from "@codemirror/view";
import { EditorState, Prec } from "@codemirror/state";
import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import { markdown } from "@codemirror/lang-markdown";
import { search, setSearchQuery, findNext as cmFindNext, findPrevious as cmFindPrevious, replaceNext as cmReplaceNext, replaceAll as cmReplaceAll } from "@codemirror/search";

const cmHost = ref(null);
let view = null;
const stateCache = new Map(); // cid -> EditorState

const chTitle = ref("");
const findQ = ref("");
const findR = ref("");
const findTotal = ref(0);
const findCur = ref(0);
let lastQuery = "";

const fontFamily = () => {
  const f = store.settings.editor.font || "serif";
  return f === "serif"
    ? '"Noto Serif CJK SC", "Source Han Serif SC", "SimSun", "宋体", serif'
    : '"Noto Sans CJK SC", "Microsoft YaHei", "PingFang SC", sans-serif';
};

function editorExtensions() {
  const s = store.settings.editor;
  const dark = store.settings.theme !== "light";
  const theme = EditorView.theme(
    {
      "&": { fontSize: `${s.font_size}px`, backgroundColor: "var(--editor)", color: "var(--text)" },
      ".cm-content": {
        fontFamily: fontFamily(),
        lineHeight: `${s.line_spacing}`,
        letterSpacing: s.font === "serif" ? "0.02em" : "0",
        caretColor: "var(--accent)",
        padding: "18px 24px",
      },
      ".cm-line": { padding: "0 2px" },
      ".cm-scroller": { fontFamily: fontFamily(), lineHeight: `${s.line_spacing}` },
      ".cm-gutters": {
        backgroundColor: "var(--editor)",
        color: "var(--text-3)",
        borderRight: "1px solid var(--border)",
        fontSize: `${Math.max(10, s.font_size - 4)}px`,
      },
      ".cm-activeLine": { backgroundColor: "var(--accent-softer)" },
      ".cm-activeLineGutter": { backgroundColor: "transparent", color: "var(--text-2)" },
      ".cm-cursor": { borderLeft: "2px solid var(--accent)" },
      ".cm-selectionBackground, &.cm-focused .cm-selectionBackground": {
        backgroundColor: "var(--accent-soft) !important",
      },
      ".cm-selectionMatch": { backgroundColor: "var(--accent-softer)" },
      "&.cm-focused": { outline: "none" },
      ".cm-placeholder": { color: "var(--text-3)" },
    },
    { dark }
  );
  const exts = [
    placeholder("从这里开始写作…（自动保存已开启）"),
    lineNumbers(),
    highlightActiveLine(),
    drawSelection(),
    highlightSpecialChars(),
    history(),
    search({ top: true }),
    theme,
    EditorView.updateListener.of(handleUpdate),
    Prec.highest(
      keymap.of([
        { key: "Mod-f", run: () => { store.findOpen = true; store.findReplace = false; return true; } },
        { key: "Mod-h", run: () => { store.findOpen = true; store.findReplace = true; return true; } },
        ...defaultKeymap,
        ...historyKeymap,
        indentWithTab,
      ])
    ),
  ];
  if (s.markdown_highlight) exts.push(markdown());
  return exts;
}

function handleUpdate(update) {
  const cid = store.activeTab;
  if (!cid) return;
  if (update.docChanged) {
    const text = update.state.doc.toString();
    store.chapters[cid] = text;
    store.wordCount = countWords(text);
    markDirty(cid);
  }
  if (update.selectionSet || update.docChanged) {
    const sel = update.state.selection.main;
    const pos = sel.head;
    const line = update.state.doc.lineAt(pos);
    store.cursorPos = { line: line.number, col: pos - line.from + 1 };
    if (sel.from !== sel.to) {
      store.selectedText = update.state.sliceDoc(sel.from, sel.to);
    } else {
      store.selectedText = "";
    }
  }
}

function activate(cid) {
  store.activeTab = cid;
  switchDoc();
}

function shortTitle(cid) {
  const t = allChapters(store.novel).find((c) => c.id === cid)?.title || "未命名";
  return t.length > 10 ? t.slice(0, 10) + "…" : t;
}

async function closeTabLocal(cid) {
  const { closeTab } = await import("../store");
  closeTab(cid);
  nextTick(() => switchDoc());
}

function openTabMenu(e, cid) {
  window.dispatchEvent(
    new CustomEvent("jinshu:contextmenu", {
      detail: {
        x: e.clientX,
        y: e.clientY,
        items: [
          { label: "保存", run: () => saveChapterNow(cid) },
          { label: "关闭", run: () => closeTabLocal(cid) },
          {
            label: "关闭其他",
            run: () => {
              for (const o of [...store.openTabs]) if (o !== cid) closeTabLocal(o);
              store.activeTab = cid;
            },
          },
        ],
      },
    })
  );
}

let lastCid = null;
function switchDoc() {
  const cid = store.activeTab;
  if (!view || !cid) return;
  // 缓存上一个标签页的状态（含撤销历史）
  if (lastCid && lastCid !== cid && store.chapters[lastCid] !== undefined) {
    stateCache.set(lastCid, view.state);
  }
  lastCid = cid;
  const cached = stateCache.get(cid);
  if (store.chapters[cid] === undefined) store.chapters[cid] = "";
  if (cached) {
    view.setState(cached);
  } else {
    const doc = store.chapters[cid] || "";
    store.wordCount = countWords(doc);
    view.setState(EditorState.create({ doc, extensions: editorExtensions() }));
  }
  store.findOpen = false;
  const title = allChapters(store.novel).find((c) => c.id === cid)?.title || "";
  chTitle.value = title;
  view.focus();
}

// 保存标题
function saveTitle() {
  const cid = store.activeTab;
  const c = allChapters(store.novel).find((x) => x.id === cid);
  if (c) {
    c.title = chTitle.value.trim() || c.title;
    api.saveNovel(store.novel);
  }
}

function saveNow() {
  saveAll().then(() => toast("已保存（加密写入本地）"));
}

// 查找
function updateFind() {
  const cid = store.activeTab;
  if (!cid) return;
  lastQuery = findQ.value;
  const text = view.state.doc.toString();
  const matches = [...text.matchAll(new RegExp(escapeReg(lastQuery), "g"))];
  findTotal.value = matches.length;
  findCur.value = 0;
  if (lastQuery) {
    view.dispatch(setSearchQuery({ search: lastQuery }));
    const cur = view.state.selection.main.head;
    const idx = matches.findIndex((m) => m.index >= cur);
    if (idx > 0) findCur.value = idx;
  }
}
function escapeReg(s) {
  return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
function findNext() {
  if (!lastQuery) return;
  view.dispatch(setSearchQuery({ search: lastQuery }));
  view.focus();
  const before = view.state.selection.main.head;
  cmFindNext(view);
  const after = view.state.selection.main.head;
  if (after !== before) findCur.value = (findCur.value + 1) % Math.max(1, findTotal.value);
}
function findPrev() {
  if (!lastQuery) return;
  view.dispatch(setSearchQuery({ search: lastQuery }));
  view.focus();
  cmFindPrevious(view);
  findCur.value = (findCur.value - 1 + findTotal.value) % Math.max(1, findTotal.value);
}
function replaceCur() {
  view.dispatch(setSearchQuery({ search: lastQuery, replace: findR.value }));
  view.focus();
  cmReplaceNext(view);
  updateFind();
}
function replaceAll() {
  view.dispatch(setSearchQuery({ search: lastQuery, replace: findR.value }));
  view.focus();
  cmReplaceAll(view);
  updateFind();
  toast("替换完成");
}

// 章节头续写
function quickContinue() {
  import("../store").then((m) => {
    store.aiPanelOpen = true;
    m.startAi("续写", prompts.buildContinue());
  });
}

// AI 插入
function onInsert(e) {
  const { cid, content } = e.detail;
  if (cid !== store.activeTab || !view) return;
  const sel = view.state.selection.main;
  view.dispatch({
    changes: { from: sel.from, insert: "\n\n" + content.trim() + "\n" },
    selection: { anchor: sel.from },
  });
  view.focus();
}

// 设置变化时重建主题
watch(
  () => [store.settings.editor.font_size, store.settings.editor.line_spacing, store.settings.editor.font, store.settings.editor.markdown_highlight, store.settings.theme],
  () => {
    if (view && store.activeTab) {
      const cached = stateCache.get(store.activeTab);
      view.setState(EditorState.create({ doc: view.state.doc.toString(), extensions: editorExtensions() }));
    }
  }
);

// 章节切换
watch(
  () => store.activeTab,
  () => switchDoc()
);

onMounted(() => {
  view = new EditorView({
    parent: cmHost.value,
    state: EditorState.create({ doc: "", extensions: editorExtensions() }),
  });
  window.addEventListener("jinshu:insert", onInsert);
  switchDoc();
});

onBeforeUnmount(() => {
  window.removeEventListener("jinshu:insert", onInsert);
  view?.destroy();
});
</script>

<style scoped>
.editor-wrap { flex: 1; display: flex; flex-direction: column; min-height: 0; }
.tabbar {
  display: flex;
  align-items: flex-end;
  gap: 3px;
  padding: 6px 8px 0;
  flex-shrink: 0;
}
.tab {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 5px 8px 5px 12px;
  border-radius: 7px 7px 0 0;
  font-size: 12.5px;
  color: var(--text-2);
  cursor: pointer;
  max-width: 180px;
  border: 1px solid transparent;
}
.tab:hover { background: var(--hover); color: var(--text); }
.tab.active { background: var(--panel); color: var(--text); border-color: var(--border); border-bottom-color: var(--panel); }
.tab-title { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.tab .dot { width: 6px; height: 6px; border-radius: 50%; background: var(--warn); flex-shrink: 0; }
.tab-close { font-size: 10px; color: var(--text-3); border-radius: 4px; padding: 1px 3px; }
.tab-close:hover { background: var(--hover); color: var(--text); }
.tabbar-grow { flex: 1; border-bottom: 1px solid var(--border); }

.chapter-head {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 7px 14px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}
.ch-title {
  border: none;
  background: var(--panel-alt);
  color: var(--text);
  font-size: 15px;
  font-weight: 600;
  padding: 5px 12px;
  border-radius: 6px;
  width: 360px;
  outline: none;
  font-family: inherit;
}
.ch-title:focus { box-shadow: 0 0 0 1px var(--accent); }
.ch-words { font-size: 12px; color: var(--text-2); }
.ch-actions { margin-left: auto; display: flex; gap: 6px; }

.findbar {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 10px;
  background: var(--panel);
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}
.find-input { width: 200px; padding: 4px 9px; font-size: 12px; }
.find-count { font-size: 12px; color: var(--text-2); min-width: 44px; }
.find-count.none { color: var(--danger); }
.cm-host { flex: 1; min-height: 0; overflow: hidden; }
</style>
