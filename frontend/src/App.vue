<template>
  <div class="app" v-if="store.ready">
    <TitleBar />
    <div class="body">
      <ActivityBar v-if="!store.focusMode" />
      <SidePanel v-if="!store.focusMode && showSidebar" />
      <main class="central">
        <Library v-if="store.activity === 'library'" />
        <StatsView v-else-if="store.activity === 'stats'" />
        <SettingsView v-else-if="store.activity === 'settings'" />
        <DetailViews v-else-if="isDetailActivity" />
        <EditorView v-else-if="store.novel && store.activeTab" />
        <div v-else class="empty" style="height:100%">
          <div class="emoji">📑</div>
          <div class="title">还没有打开章节</div>
          <div class="sub">在左侧「章节」面板选择或新建一个章节；没有作品时先到书库新建一部</div>
        </div>
      </main>
      <AIPanel v-if="!store.focusMode" />
    </div>
    <StatusBar />
    <Palette v-if="store.paletteOpen" />
    <Modal v-if="store.dialog" />
    <ContextMenu />
    <div class="toast" v-if="store.toast" :class="{ ok: store.toast.ok, err: !store.toast.ok }">
      {{ store.toast.msg }}
    </div>
  </div>
  <div v-else class="splash">
    <div class="splash-logo">📖</div>
    <div class="splash-title">锦书</div>
  </div>
</template>

<script setup>
import { computed } from "vue";
import { store } from "./store";
import TitleBar from "./components/TitleBar.vue";
import ActivityBar from "./components/ActivityBar.vue";
import SidePanel from "./components/SidePanel.vue";
import StatusBar from "./components/StatusBar.vue";
import EditorView from "./components/EditorView.vue";
import AIPanel from "./components/AIPanel.vue";
import Palette from "./components/Palette.vue";
import Modal from "./components/Modal.vue";
import ContextMenu from "./components/ContextMenu.vue";
import Library from "./views/Library.vue";
import StatsView from "./views/StatsView.vue";
import SettingsView from "./views/SettingsView.vue";
import DetailViews from "./views/DetailViews.vue";

const showSidebar = computed(
  () =>
    store.sidebarOpen &&
    store.novel &&
    ["chapters", "outline", "characters", "world", "timeline", "tasks", "search"].includes(store.activity)
);
const isDetailActivity = computed(
  () =>
    store.novel &&
    ["characters", "world", "timeline", "tasks", "outline"].includes(store.activity)
);
</script>

<style scoped>
.app {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--editor);
}
.body {
  display: flex;
  flex: 1;
  min-height: 0;
}
.central {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  background: var(--editor);
  position: relative;
}
.splash {
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  background: var(--editor);
}
.splash-logo { font-size: 46px; }
.splash-title { font-size: 22px; color: var(--text); }
.toast {
  position: fixed;
  right: 20px;
  bottom: 40px;
  z-index: 300;
  padding: 9px 16px;
  border-radius: 8px;
  font-size: 12.5px;
  color: #fff;
  box-shadow: var(--shadow);
  animation: slideUp 0.2s ease;
  max-width: 420px;
}
.toast.ok { background: var(--ok); }
.toast.err { background: var(--danger); }
</style>
