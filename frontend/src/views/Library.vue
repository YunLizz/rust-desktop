<template>
  <div class="library">
    <!-- 欢迎页（空书库） -->
    <div v-if="!store.library.length" class="welcome">
      <div class="w-logo">📖</div>
      <div class="w-title">锦书</div>
      <div class="w-sub">为中文小说创作而生的本地编辑器 · 数据加密存储</div>
      <div class="w-cards">
        <div class="w-card" @click="store.dialog = { kind: 'newNovel' }">
          <div class="wc-icon">📝</div>
          <div class="wc-title">新建小说</div>
          <div class="wc-sub">创建一部新作品</div>
        </div>
        <div class="w-card" @click="store.activity = 'library'; refresh()">
          <div class="wc-icon">📚</div>
          <div class="wc-title">打开书库</div>
          <div class="wc-sub">浏览本地作品库</div>
        </div>
        <div class="w-card" @click="store.dialog = { kind: 'import' }">
          <div class="wc-icon">🔑</div>
          <div class="wc-title">导入备份</div>
          <div class="wc-sub">从 .jsb 加密备份恢复</div>
        </div>
      </div>
      <div v-if="store.settings?.recent?.length" class="w-recent">
        <div class="wr-label">最近打开</div>
        <div class="wr-list">
          <span v-for="r in store.settings.recent" :key="r.id" class="wr-item" @click="openNovel(r.id)">{{ r.title }}</span>
        </div>
      </div>
    </div>

    <!-- 书库 -->
    <template v-else>
      <div class="lib-head">
        <div class="lib-title">📚 我的书库</div>
        <div class="lib-actions">
          <button class="btn" @click="refresh">刷新</button>
          <button class="btn" @click="store.dialog = { kind: 'import' }">导入 .jsb</button>
          <button class="btn primary" @click="store.dialog = { kind: 'newNovel' }">＋ 新建小说</button>
        </div>
      </div>
      <div class="lib-grid">
        <div
          v-for="m in store.library"
          :key="m.id"
          class="book-card"
          @dblclick="openNovel(m.id)"
          @contextmenu.prevent="openMenu($event, m)"
        >
          <div class="bc-cover">📖</div>
          <div class="bc-info">
            <div class="bc-title">{{ m.title }}</div>
            <div class="bc-author" v-if="m.author">{{ m.author }}</div>
            <div class="bc-meta">{{ m.chapter_count }} 章 · {{ m.total_words }} 字</div>
            <div class="bc-time">更新于 {{ fmtTime(m.updated_at) }}</div>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup>
import { store, openNovel } from "../store";
import { api } from "../api";

async function refresh() {
  store.library = await api.listNovels();
}

function fmtTime(ts) {
  if (!ts) return "-";
  const d = new Date(ts * 1000);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")} ${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`;
}

function openMenu(e, m) {
  window.dispatchEvent(
    new CustomEvent("jinshu:contextmenu", {
      detail: {
        x: e.clientX,
        y: e.clientY,
        items: [
          { label: "打开", run: () => openNovel(m.id) },
          { label: "导出", run: () => { store.novel ? null : openNovel(m.id).then(() => (store.dialog = { kind: "export" })); } },
          { label: "删除", danger: true, run: () => (store.dialog = { kind: "deleteNovel", payload: { id: m.id, title: m.title } }) },
        ],
      },
    })
  );
}
</script>

<style scoped>
.library { flex: 1; overflow-y: auto; }
.welcome {
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
  padding-bottom: 40px;
}
.w-logo { font-size: 52px; margin-bottom: 6px; }
.w-title { font-size: 30px; font-weight: 700; }
.w-sub { font-size: 13.5px; color: var(--text-2); margin-bottom: 26px; }
.w-cards { display: flex; gap: 26px; }
.w-card {
  width: 210px;
  height: 110px;
  border-radius: 12px;
  border: 1px solid var(--border);
  background: var(--panel-alt);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 3px;
  cursor: pointer;
  transition: border-color 0.15s, transform 0.1s, background 0.15s;
}
.w-card:hover { border-color: var(--accent); background: var(--hover); transform: translateY(-2px); }
.wc-icon { font-size: 22px; }
.wc-title { font-size: 14px; font-weight: 600; }
.wc-sub { font-size: 11.5px; color: var(--text-3); }
.w-recent { margin-top: 34px; text-align: center; }
.wr-label { font-size: 12px; color: var(--text-3); margin-bottom: 8px; }
.wr-list { display: flex; gap: 8px; flex-wrap: wrap; justify-content: center; }
.wr-item { font-size: 12px; color: var(--text-2); cursor: pointer; padding: 3px 10px; border-radius: 12px; border: 1px solid var(--border); }
.wr-item:hover { color: var(--accent); border-color: var(--accent); }

.lib-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 18px 22px 12px;
}
.lib-title { font-size: 19px; font-weight: 700; }
.lib-actions { display: flex; gap: 8px; }
.lib-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  padding: 8px 22px 30px;
}
.book-card {
  width: 250px;
  height: 132px;
  border-radius: 12px;
  border: 1px solid var(--border);
  background: var(--panel);
  display: flex;
  cursor: pointer;
  overflow: hidden;
  transition: border-color 0.15s, transform 0.1s;
}
.book-card:hover { border-color: var(--accent); transform: translateY(-2px); }
.bc-cover {
  width: 54px;
  height: 100%;
  background: var(--accent-soft);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 24px;
  flex-shrink: 0;
}
.bc-info { padding: 13px 14px; display: flex; flex-direction: column; gap: 3px; min-width: 0; }
.bc-title { font-size: 14.5px; font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.bc-author { font-size: 11.5px; color: var(--text-2); }
.bc-meta { font-size: 11px; color: var(--text-3); margin-top: auto; }
.bc-time { font-size: 10.5px; color: var(--text-3); }
</style>
