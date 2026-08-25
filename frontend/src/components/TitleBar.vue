<template>
  <header class="titlebar" data-tauri-drag-region>
    <div class="brand" data-tauri-drag-region>
      <span class="logo">📖</span>
      <span class="name">锦书</span>
      <span class="sep">·</span>
      <span class="novel" v-if="store.novel">{{ store.novel.meta.title }}</span>
      <span class="dirty-dot" v-if="Object.keys(store.dirty).length" title="有未保存修改"></span>
    </div>
    <div class="controls" v-if="isCustomTitlebar">
      <button class="ctl" title="最小化" @click="minimize"><Icon name="minus" :size="14" /></button>
      <button class="ctl" :title="maximized ? '还原' : '最大化'" @click="toggleMax"><Icon :name="maximized ? 'copy' : 'square'" :size="13" /></button>
      <button class="ctl close" title="关闭" @click="close"><Icon name="close" :size="14" /></button>
    </div>
  </header>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from "vue";
import { store } from "../store";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Icon from "./Icon.vue";

const isCustomTitlebar = !window.__TAURI_INTERNALS__?.metadata?.config?.app?.windows?.[0]?.decorations !== false;
const maximized = ref(false);
let appWindow = null;

async function initWindow() {
  try {
    appWindow = getCurrentWindow();
    maximized.value = await appWindow.isMaximized();
    appWindow.onResized(() => {
      appWindow.isMaximized().then((m) => (maximized.value = m));
    });
  } catch (e) {
    /* 浏览器调试模式 */
  }
}
const minimize = () => appWindow?.minimize();
const toggleMax = () => appWindow?.toggleMaximize();
const close = () => appWindow?.close();

onMounted(initWindow);
</script>

<style scoped>
.titlebar {
  height: 40px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--chrome);
  border-bottom: 1px solid var(--border);
  user-select: none;
}
.brand {
  display: flex;
  align-items: center;
  gap: 8px;
  padding-left: 14px;
  height: 100%;
}
.logo { font-size: 15px; }
.name { font-size: 14px; font-weight: 600; color: var(--text); }
.sep { color: var(--text-3); }
.novel { font-size: 12.5px; color: var(--text-2); max-width: 300px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.dirty-dot { width: 6px; height: 6px; border-radius: 50%; background: var(--warn); }
.controls { display: flex; height: 100%; }
.ctl {
  width: 44px;
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
</style>
