<template>
  <Teleport to="body">
    <div v-if="menu" class="ctx-overlay" @mousedown.self="menu = null" @contextmenu.prevent="menu = null">
      <div class="popup ctx" :style="{ left: menu.x + 'px', top: menu.y + 'px' }">
        <button
          v-for="(item, i) in menu.items"
          :key="i"
          class="ctx-item"
          :class="{ danger: item.danger }"
          @click="run(item)"
        >
          {{ item.label }}
        </button>
      </div>
    </div>
  </Teleport>
</template>

<script setup>
import { ref, onMounted, onBeforeUnmount } from "vue";

const menu = ref(null);

function onMenu(e) {
  const items = e.detail.items;
  if (!items?.length) return;
  menu.value = { x: Math.min(e.detail.x, window.innerWidth - 180), y: Math.min(e.detail.y, window.innerHeight - items.length * 30 - 20), items };
}
function run(item) {
  menu.value = null;
  item.run?.();
}
onMounted(() => window.addEventListener("jinshu:contextmenu", onMenu));
onBeforeUnmount(() => window.removeEventListener("jinshu:contextmenu", onMenu));
</script>

<style scoped>
.ctx-overlay { position: fixed; inset: 0; z-index: 200; }
.ctx { position: fixed; min-width: 160px; padding: 5px; z-index: 201; }
.ctx-item {
  display: block;
  width: 100%;
  text-align: left;
  padding: 6px 10px;
  border: none;
  background: transparent;
  border-radius: 5px;
  color: var(--text);
  font-size: 12.5px;
  cursor: pointer;
  font-family: inherit;
}
.ctx-item:hover { background: var(--accent-soft); }
.ctx-item.danger { color: var(--danger); }
.ctx-item.danger:hover { background: var(--danger-soft); }
</style>
