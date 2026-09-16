<template>
  <div class="app" v-if="store.ready">
    <!-- 顶部 Ribbon（含窗口控制） -->
    <Ribbon />

    <!-- 全屏视图：书库 / 统计 / 设置 / 关系网 -->
    <div class="fullview" v-if="store.fullView">
      <div class="fv-bar">
        <button class="fv-back" @click="store.fullView = null">← 返回写作台</button>
        <span class="fv-title">{{ fullViewName }}</span>
        <span class="fv-sub" v-if="store.novel">《{{ store.novel.meta.title }}》</span>
      </div>
      <main class="fv-body">
        <Library v-if="store.fullView === 'library'" />
        <StatsView v-else-if="store.fullView === 'stats'" />
        <SettingsView v-else-if="store.fullView === 'settings'" />
        <RelationGraph v-else-if="store.fullView === 'graph'" />
      </main>
    </div>

    <!-- 写作台：顶栏横向时间轴 + 三栏 -->
    <template v-else-if="store.novel">
      <!-- 剪辑式横向时间轴（可折叠） -->
      <TimelineTrack v-if="!store.focusMode" />

      <div class="body">
        <template v-if="!store.focusMode">
          <LeftPane v-if="store.leftOpen" />
          <DragHandle
            v-if="store.leftOpen"
            :get="() => store.leftWidth"
            :set="(w) => (store.leftWidth = w)"
            :dir="1"
            :min="220"
            :max="560"
            :fallback="300"
          />
        </template>

        <main class="central">
          <EditorView v-if="store.activeTab" />
          <div v-else class="empty">
            <div class="emoji">📑</div>
            <div class="title">还没有打开章节</div>
            <div class="sub">在左侧章节树点击一个章节开始写作</div>
          </div>
        </main>

        <template v-if="!store.focusMode">
          <DragHandle
            v-if="store.rightOpen"
            :get="() => store.rightWidth"
            :set="(w) => (store.rightWidth = w)"
            :dir="-1"
            :min="280"
            :max="640"
            :fallback="400"
          />
          <RightPane v-if="store.rightOpen" />
        </template>
      </div>
    </template>

    <!-- 无作品：欢迎页 -->
    <div class="body" v-else>
      <main class="central">
        <Library />
      </main>
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
import Ribbon from "./components/Ribbon.vue";
import LeftPane from "./components/LeftPane.vue";
import RightPane from "./components/RightPane.vue";
import DragHandle from "./components/DragHandle.vue";
import EditorView from "./components/EditorView.vue";
import StatusBar from "./components/StatusBar.vue";
import Palette from "./components/Palette.vue";
import Modal from "./components/Modal.vue";
import ContextMenu from "./components/ContextMenu.vue";
import Library from "./views/Library.vue";
import StatsView from "./views/StatsView.vue";
import SettingsView from "./views/SettingsView.vue";
import RelationGraph from "./views/RelationGraph.vue";
import TimelineTrack from "./components/TimelineTrack.vue";

const fullViewName = computed(
  () => ({ library: "书库", stats: "写作统计", settings: "设置", graph: "人物关系网" }[store.fullView] || "")
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
/* 全屏视图 */
.fullview {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  background: var(--editor);
}
.fv-bar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 14px;
  border-bottom: 1px solid var(--border);
  background: var(--panel);
  flex-shrink: 0;
}
.fv-back {
  border: 1px solid var(--border);
  background: var(--panel-alt);
  color: var(--text);
  font-size: 12px;
  padding: 4px 12px;
  border-radius: 6px;
  cursor: pointer;
  font-family: inherit;
  transition: border-color 0.12s, color 0.12s;
}
.fv-back:hover { border-color: var(--accent); color: var(--accent); }
.fv-title { font-size: 14.5px; font-weight: 600; color: var(--text); }
.fv-sub { font-size: 12px; color: var(--text-3); }
.fv-body { flex: 1; min-height: 0; display: flex; flex-direction: column; }

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
