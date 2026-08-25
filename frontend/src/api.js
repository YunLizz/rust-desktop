// Tauri 命令封装
import { invoke } from "@tauri-apps/api/core";

export const api = {
  initInfo: () => invoke("init_info"),
  listNovels: () => invoke("list_novels"),
  loadNovel: (id) => invoke("load_novel", { id }),
  createNovel: (p) => invoke("create_novel", p),
  saveNovel: (novel) => invoke("save_novel", { novel }),
  saveChapter: (novelId, cid, text) => invoke("save_chapter", { novelId, cid, text }),
  deleteChapter: (novelId, cid) => invoke("delete_chapter", { novelId, cid }),
  deleteNovel: (id) => invoke("delete_novel", { id }),
  loadSettings: () => invoke("load_settings"),
  saveSettings: (settings) => invoke("save_settings", { settings }),
  exportWork: (p) => invoke("export_work", p),
  importJsb: (path, password) => invoke("import_jsb", { path, password }),
  openDir: (path) => invoke("open_dir", { path }),
  aiStart: (cfg, messages) => invoke("ai_start", { cfg, messages }),
  aiCancel: () => invoke("ai_cancel"),
  aiTest: (cfg) => invoke("ai_test", { cfg }),
};
