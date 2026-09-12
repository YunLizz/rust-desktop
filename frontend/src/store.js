// 全局响应式状态与业务动作
import { reactive } from "vue";
import { api } from "./api";

export const store = reactive({
  ready: false,
  dataDir: "",
  keyFp: "",
  settings: null,

  library: [],
  novel: null,
  chapters: {}, // cid -> text（内存态，EditorView 同步）
  dirty: {}, // cid -> true
  openTabs: [],
  activeTab: null,

  // ======== 新布局模型（Ribbon + 三栏可拖拽）========
  ribbonTab: "writing",   // Ribbon 选项卡: writing | lore | tools | view
  leftTree: "chapters",   // 左栏 4 树: chapters | outline | characters | world
  rightTool: null,        // 右栏工具: null(工具格) | namer | map | graph | timeline | tasks
  timelineOpen: false,    // 顶栏横向时间轴是否展开
  mapPos: {},             // 地图节点位置
  fullView: null,         // 全屏视图（覆盖三栏）: library | stats | settings | graph
  leftWidth: 300,         // 左栏宽度（可拖拽）
  rightWidth: 400,        // 右栏宽度（可拖拽）
  leftOpen: true,
  rightOpen: true,
  aiPanelOpen: true,      // 保留：AI 功能恢复时使用
  focusMode: false,
  paletteOpen: false,
  paletteQuery: "",
  findOpen: false,
  findReplace: false,

  selChar: null,
  selLoc: null,
  selEvent: null,
  selOutline: null,
  selChain: "all",
  selTask: null,
  showRelCanvas: false,
  canvasPos: {},

  dialog: null,
  toast: null,
  toastOk: true,

  selectedText: "",
  cursorPos: null,
  wordCount: 0,
  saveTimer: null,
  lastAutosave: 0,
});

export function toast(msg, ok = true) {
  store.toast = { msg, ok, t: Date.now() };
  setTimeout(() => {
    if (store.toast && Date.now() - store.toast.t > 3000) store.toast = null;
  }, 3200);
}

export function today() {
  const d = new Date();
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}

// ---------- 小说 ----------
export async function openNovel(id) {
  try {
    const { novel, chapters } = await api.loadNovel(id);
    store.novel = novel;
    store.chapters = {};
    store.dirty = {};
    store.openTabs = [];
    store.activeTab = null;
    for (const c of chapters) store.chapters[c.id] = c.text;
    store.fullView = null;
    store.leftOpen = true;
    touchRecent(id, novel.meta.title);
    // 恢复上次打开的章节
    const last = store.settings.lastChapter?.[id];
    if (last && store.chapters[last] !== undefined) openTab(last);
    else {
      // 打开第一章
      const first = allChapters(novel)[0];
      if (first) openTab(first.id);
    }
  } catch (e) {
    toast(String(e), false);
  }
}

function touchRecent(id, title) {
  const s = store.settings;
  s.recent = [{ id, title, opened_at: Date.now() }, ...(s.recent || []).filter((r) => r.id !== id)].slice(0, 8);
  s.last_novel_id = id;
  saveSettings();
}

export async function createNovel(form) {
  try {
    const id = await api.createNovel(form);
    store.library = await api.listNovels();
    toast("小说已创建（加密存储）");
    await openNovel(id);
    return true;
  } catch (e) {
    toast(String(e), false);
    return false;
  }
}

export async function closeNovel() {
  await saveAll();
  store.novel = null;
  store.chapters = {};
  store.openTabs = [];
  store.activeTab = null;
  store.settings.last_novel_id = null;
  saveSettings();
  store.library = await api.listNovels();
  store.fullView = "library";
}

// ---------- 标签页 ----------
export function openTab(cid) {
  if (!store.openTabs.includes(cid)) store.openTabs.push(cid);
  store.activeTab = cid;
  store.fullView = null;
  if (store.settings) {
    store.settings.lastChapter = store.settings.lastChapter || {};
    store.settings.lastChapter[store.novel?.meta?.id] = cid;
    saveSettings();
  }
}

export function closeTab(cid) {
  saveChapterNow(cid);
  store.openTabs = store.openTabs.filter((c) => c !== cid);
  delete store.chapters[cid];
  delete store.dirty[cid];
  if (store.activeTab === cid) store.activeTab = store.openTabs[store.openTabs.length - 1] || null;
}

export function markDirty(cid) {
  store.dirty[cid] = true;
  store.novel.meta.updated_at = Date.now() / 1000 | 0;
  scheduleSave();
}

// ---------- 保存 ----------
export function scheduleSave() {
  if (store.saveTimer) clearTimeout(store.saveTimer);
  store.saveTimer = setTimeout(() => saveAll(), 2000);
}

export async function saveChapterNow(cid) {
  if (!store.novel || !store.dirty[cid]) return;
  const text = store.chapters[cid];
  if (text === undefined) return;
  try {
    const r = await api.saveChapter(store.novel.meta.id, cid, text);
    delete store.dirty[cid];
    if (store.novel) {
      store.novel.stats = r.stats;
      store.novel.meta.total_words = r.total_words;
      const cm = allChapters(store.novel).find((c) => c.id === cid);
      if (cm) cm.words = r.words;
    }
  } catch (e) {
    toast(`保存失败：${e}`, false);
  }
}

export async function saveAll() {
  if (store.saveTimer) {
    clearTimeout(store.saveTimer);
    store.saveTimer = null;
  }
  const cids = Object.keys(store.dirty);
  for (const cid of cids) await saveChapterNow(cid);
  if (store.novel) {
    try {
      await api.saveNovel(store.novel);
      store.novel.meta.updated_at = Date.now() / 1000 | 0;
    } catch (e) {
      toast(`保存失败：${e}`, false);
    }
  }
}

// ---------- 设置 ----------
export async function saveSettings() {
  if (!store.settings) return;
  try {
    await api.saveSettings(store.settings);
  } catch (e) {
    console.warn("save settings failed", e);
  }
}

export function applyTheme() {
  const s = store.settings;
  if (!s) return;
  document.documentElement.dataset.theme = s.theme === "light" ? "light" : "dark";
  const [r, g, b] = s.accent || [110, 139, 255];
  document.documentElement.style.setProperty("--accent", `rgb(${r}, ${g}, ${b})`);
  document.documentElement.style.setProperty("--accent-soft", `rgba(${r}, ${g}, ${b}, 0.15)`);
  document.documentElement.style.setProperty("--accent-softer", `rgba(${r}, ${g}, ${b}, 0.07)`);
  document.documentElement.style.setProperty("--accent-strong", `rgb(${Math.min(255, r + 25)}, ${Math.min(255, g + 25)}, ${Math.min(255, b + 25)})`);
}

// ---------- 辅助 ----------
export function allChapters(novel) {
  if (!novel) return [];
  return (novel.volumes || []).flatMap((v) => v.chapters || []);
}

export function chapterTitle(cid) {
  return allChapters(store.novel)?.find((c) => c.id === cid)?.title || "未命名";
}

export function countWords(s) {
  let n = 0;
  let inWord = false;
  for (const ch of s) {
    const code = ch.codePointAt(0);
    const cjk = (code >= 0x4e00 && code <= 0x9fff) || (code >= 0x3400 && code <= 0x4dbf) ||
      (code >= 0x20000 && code <= 0x2a6df) || (code >= 0xf900 && code <= 0xfaff) ||
      (code >= 0x3000 && code <= 0x303f) || (code >= 0xff00 && code <= 0xffef) ||
      (code >= 0x3040 && code <= 0x30ff);
    if (cjk) {
      n++;
      inWord = false;
    } else if (/[A-Za-z0-9]/.test(ch)) {
      if (!inWord) {
        n++;
        inWord = true;
      }
    } else inWord = false;
  }
  return n;
}

export function truncate(s, n) {
  return s.length > n ? s.slice(0, n) + "…" : s;
}
