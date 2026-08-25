import { createApp } from "vue";
import App from "./App.vue";
import "./styles/theme.css";
import { store, applyTheme } from "./store";
import { api } from "./api";
import { listen } from "@tauri-apps/api/event";

async function init() {
  try {
    const info = await api.initInfo();
    store.dataDir = info.data_dir;
    store.keyFp = info.key_fp;
    store.settings = info.settings;
    store.library = await api.listNovels();
    store.ready = true;
    applyTheme();
    // 恢复上次打开的小说
    if (info.settings.last_novel_id) {
      try {
        const { openNovel } = await import("./store");
        await openNovel(info.settings.last_novel_id);
      } catch (e) {
        console.warn("恢复上次作品失败", e);
        store.library = await api.listNovels();
      }
    }
  } catch (e) {
    document.getElementById("app").innerHTML =
      `<div style="padding:40px;color:#fb464c;font-family:sans-serif">初始化失败：${e}<br>若为首次运行请确认数据目录可写。</div>`;
  }
}

// AI 事件
listen("ai-chunk", (e) => {
  store.aiStreamText += e.payload;
});
listen("ai-done", () => {
  const content = store.aiStreamText;
  if (content.trim()) {
    store.aiMsgs.push({ role: "assistant", content, action: store.aiAction });
  }
  store.aiStreaming = false;
  store.aiStreamText = "";
  store.aiAction = "";
});
listen("ai-error", (e) => {
  store.aiMsgs.push({ role: "error", content: e.payload, action: store.aiAction });
  store.aiStreaming = false;
  store.aiStreamText = "";
  store.aiAction = "";
});
listen("ai-test-result", (e) => {
  const p = e.payload;
  store.aiTestResult = p && p.Ok ? [true, p.Ok] : p && p.Err ? [false, p.Err] : [false, String(p)];
  setTimeout(() => (store.aiTestResult = null), 6000);
});

// 禁用 WebView2 默认右键菜单（后退/刷新等），改用应用内自定义菜单
document.addEventListener("contextmenu", (e) => e.preventDefault());

// 全局快捷键（依赖活动元素，简单处理）
window.addEventListener("keydown", async (e) => {
  const mod = e.ctrlKey || e.metaKey;
  const tag = document.activeElement?.tagName;
  const inEditor = !!document.querySelector(".cm-editor")?.contains(document.activeElement);
  if (mod && e.key === "s") {
    e.preventDefault();
    const { saveAll, toast } = await import("./store");
    await saveAll();
    toast("已保存（加密写入本地）");
  }
  if (mod && e.key === "p" && !inEditor) {
    e.preventDefault();
    store.paletteOpen = !store.paletteOpen;
  }
  if (mod && e.key === "b") {
    e.preventDefault();
    store.sidebarOpen = !store.sidebarOpen;
  }
  if (mod && e.key === "j") {
    e.preventDefault();
    store.aiPanelOpen = !store.aiPanelOpen;
  }
  if (mod && e.key === "=") {
    e.preventDefault();
    store.settings.editor.font_size = Math.min(32, (store.settings.editor.font_size || 17) + 1);
  }
  if (mod && e.key === "-") {
    e.preventDefault();
    store.settings.editor.font_size = Math.max(10, (store.settings.editor.font_size || 17) - 1);
  }
  if (mod && e.key === "n" && !inEditor) {
    e.preventDefault();
    store.dialog = { kind: "newNovel" };
  }
  if (e.key === "Escape") {
    store.paletteOpen = false;
    store.findOpen = false;
    if (store.dialog) store.dialog = null;
  }
});

init();
window.__store = store; // 调试用
createApp(App).mount("#app");
