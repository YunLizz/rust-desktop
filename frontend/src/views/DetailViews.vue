<template>
  <div class="detail">
    <!-- ========== 人物卡详情 ========== -->
    <template v-if="store.activity === 'characters'">
      <div v-if="!store.showRelCanvas" class="d-body">
        <div class="d-head">
          <span class="d-title">👤 人物卡</span>
          <span class="d-hint">（所有修改自动保存）</span>
          <div class="d-actions">
            <button class="btn sm" @click="store.showRelCanvas = true">🧩 关系网</button>
            <button class="btn sm" @click="aiChar">✨ AI 完善人设</button>
            <button class="btn sm danger" @click="delChar">🗑 删除</button>
          </div>
        </div>
        <div class="d-form" v-if="char">
          <div class="row2">
            <label>名字</label>
            <input class="input" v-model="char.name" @change="save" />
            <label style="margin-left:16px">定位</label>
            <select class="input" style="width:120px" v-model="char.role" @change="save">
              <option v-for="r in ['主角', '重要配角', '配角', '反派', '其他']" :key="r" :value="r">{{ r }}</option>
            </select>
          </div>
          <div class="field"><label>外貌</label><textarea class="input" rows="2" v-model="char.appearance" @change="save"></textarea></div>
          <div class="field"><label>性格</label><textarea class="input" rows="2" v-model="char.personality" @change="save"></textarea></div>
          <div class="field"><label>背景经历</label><textarea class="input" rows="3" v-model="char.background" @change="save"></textarea></div>
          <div class="field"><label>目标与欲望</label><textarea class="input" rows="2" v-model="char.goals" @change="save"></textarea></div>
          <div class="field"><label>备注</label><textarea class="input" rows="2" v-model="char.notes" @change="save"></textarea></div>

          <div class="field">
            <label>关系</label>
            <div v-for="(r, i) in char.relationships" :key="i" class="rel-row">
              <span class="rel-name">↔ {{ r.target_name }}</span>
              <span class="rel-text">{{ r.relation }}</span>
              <button class="icon-btn" @click="delRel(i)">✕</button>
            </div>
            <div class="rel-add">
              <select class="input" style="width:150px" v-model="relTarget">
                <option value="">选择对象…</option>
                <option v-for="o in others" :key="o.id" :value="o.id">{{ o.name }}</option>
              </select>
              <input class="input" style="flex:1" v-model="relText" placeholder="关系（如：兄妹 / 宿敌）" @keydown.enter="addRel" />
              <button class="btn sm" @click="addRel">添加</button>
            </div>
          </div>
        </div>
        <div v-else class="empty">
          <div class="emoji">👥</div>
          <div class="title">在左侧选择一个人物</div>
        </div>
      </div>
      <!-- 关系网画布 -->
      <div v-else class="canvas-wrap">
        <div class="d-head">
          <span class="d-title">🧩 人物关系网</span>
          <span class="d-hint">拖动节点调整布局</span>
          <div class="d-actions">
            <button class="btn sm" @click="resetCanvas">重置布局</button>
            <button class="btn sm" @click="store.showRelCanvas = false">返回人物卡</button>
          </div>
        </div>
        <div class="canvas" ref="canvas">
          <svg :width="cw" :height="ch" class="canvas-svg">
            <line
              v-for="(edge, i) in edges"
              :key="i"
              :x1="edge.a.x" :y1="edge.a.y" :x2="edge.b.x" :y2="edge.b.y"
              stroke="var(--border-strong)" stroke-width="1.5"
            />
            <text
              v-for="(edge, i) in edges"
              :key="'t' + i"
              :x="(edge.a.x + edge.b.x) / 2" :y="(edge.a.y + edge.b.y) / 2 - 6"
              text-anchor="middle" font-size="11" fill="var(--text-3)"
            >{{ edge.relation }}</text>
          </svg>
          <div
            v-for="c in store.novel?.characters"
            :key="c.id"
            class="cnode"
            :class="{ sel: c.id === store.selChar }"
            :style="{ left: (pos(c.id)?.x || 0) + 'px', top: (pos(c.id)?.y || 0) + 'px' }"
            @mousedown="startDrag(c.id, $event)"
            @click="store.selChar = c.id"
          >{{ c.name }}</div>
        </div>
      </div>
    </template>

    <!-- ========== 世界观详情 ========== -->
    <template v-else-if="store.activity === 'world'">
      <div class="d-body" v-if="loc">
        <div class="d-head">
          <span class="d-title">🗺️ 地点设定</span>
          <div class="d-actions">
            <button class="btn sm danger" @click="delLoc">🗑 删除</button>
          </div>
        </div>
        <div class="row2">
          <label>名称</label>
          <input class="input" v-model="loc.name" @change="save" />
          <label style="margin-left:16px">类别</label>
          <select class="input" style="width:120px" v-model="loc.kind" @change="save">
            <option v-for="k in ['国家', '城市', '地区', '建筑', '异界', '其他']" :key="k" :value="k">{{ k }}</option>
          </select>
        </div>
        <div class="field"><label>描述</label><textarea class="input" rows="10" v-model="loc.description" @change="save"></textarea></div>
      </div>
      <div v-else class="empty">
        <div class="emoji">🗺️</div>
        <div class="title">在左侧选择一个地点</div>
      </div>
    </template>

    <!-- ========== 时间线详情 ========== -->
    <template v-else-if="store.activity === 'timeline'">
      <div class="d-body" v-if="ev">
        <div class="d-head">
          <span class="d-title">⏱️ 事件</span>
          <div class="d-actions">
            <button class="btn sm danger" @click="delEv">🗑 删除</button>
          </div>
        </div>
        <div class="row2">
          <label>标题</label>
          <input class="input" v-model="ev.title" @change="save" />
        </div>
        <div class="row2">
          <label>时间</label>
          <input class="input" v-model="ev.time" placeholder="如：第一卷 第3章 前夜" @change="save" />
        </div>
        <div class="field"><label>描述</label><textarea class="input" rows="8" v-model="ev.description" @change="save"></textarea></div>
        <div class="row2">
          <label>关联章节</label>
          <select class="input" style="width:200px" v-model="ev.chapter_id" @change="save">
            <option :value="null">无</option>
            <option v-for="c in chapters" :key="c.id" :value="c.id">{{ c.title }}</option>
          </select>
        </div>
      </div>
      <div v-else class="empty">
        <div class="emoji">⏱️</div>
        <div class="title">在左侧选择或新建一个事件</div>
      </div>
    </template>

    <!-- ========== 任务看板 ========== -->
    <template v-else-if="store.activity === 'tasks'">
      <div class="d-head" style="padding: 14px 22px 0">
        <span class="d-title">🎯 任务看板</span>
        <span class="d-hint">（待办 → 进行中 → 已完成，双击卡片切换状态）</span>
        <div class="d-actions">
          <button class="btn sm" @click="newTask">＋ 新建任务</button>
        </div>
      </div>
      <div class="board">
        <div v-for="(col, ci) in cols" :key="ci" class="bcol" :style="{ '--colc': col.color }">
          <div class="bcol-head">{{ col.name }}（{{ colTasks(ci).length }}）</div>
          <div class="bcol-body">
            <div
              v-for="t in colTasks(ci)"
              :key="t.id"
              class="task-card"
              @dblclick="cycle(t)"
              @contextmenu.prevent="openTaskMenu($event, t)"
            >
              <div class="tc-title">{{ t.title }}</div>
              <div class="tc-desc" v-if="t.description">{{ t.description }}</div>
              <div class="tc-ops">
                <button v-if="ci > 0" class="tco" title="左移" @click="moveTask(t, ci - 1)">◀</button>
                <button v-if="ci < 2" class="tco" title="右移" @click="moveTask(t, ci + 1)">▶</button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </template>

    <!-- ========== 大纲详情 ========== -->
    <template v-else-if="store.activity === 'outline'">
      <div class="d-body" v-if="outNode">
        <div class="d-head">
          <span class="d-title">🗂️ 大纲节点</span>
          <span class="d-hint">类型：{{ outNode.kind }}</span>
        </div>
        <div class="row2">
          <label>标题</label>
          <input class="input" v-model="outNode.title" @change="save" />
        </div>
        <div class="field"><label>内容要点</label><textarea class="input" rows="10" v-model="outNode.content" @change="save"></textarea></div>
      </div>
      <div v-else class="empty">
        <div class="emoji">🗂️</div>
        <div class="title">在左侧选中一个大纲节点</div>
        <div class="sub">或点击「✨ 生成大纲」由 AI 创建</div>
      </div>
    </template>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onBeforeUnmount, watch } from "vue";
import { store, toast , allChapters } from "../store";
import { api } from "../api";
import * as prompts from "../prompts";

const char = computed(() => store.novel?.characters?.find((c) => c.id === store.selChar));
const loc = computed(() => store.novel?.locations?.find((l) => l.id === store.selLoc));
const ev = computed(() => store.novel?.timeline?.find((e) => e.id === store.selEvent));
const chapters = computed(() => allChapters(store.novel) || []);
const others = computed(() => (store.novel?.characters || []).filter((c) => c.id !== store.selChar));

const relTarget = ref("");
const relText = ref("");

function save() {
  api.saveNovel(store.novel);
}
function delChar() {
  if (!char.value) return;
  store.novel.characters = store.novel.characters.filter((c) => c.id !== char.value.id);
  store.selChar = null;
  save();
}
function delLoc() {
  if (!loc.value) return;
  store.novel.locations = store.novel.locations.filter((l) => l.id !== loc.value.id && l.parent_id !== loc.value.id);
  store.selLoc = null;
  save();
}
function delEv() {
  if (!ev.value) return;
  store.novel.timeline = store.novel.timeline.filter((e) => e.id !== ev.value.id);
  store.selEvent = null;
  save();
}
function delRel(i) {
  char.value.relationships.splice(i, 1);
  save();
}
function addRel() {
  if (!relTarget.value) return;
  const t = others.value.find((o) => o.id === relTarget.value);
  char.value.relationships.push({ target_id: t.id, target_name: t.name, relation: relText.value.trim(), note: "" });
  relTarget.value = "";
  relText.value = "";
  save();
}
function aiChar() {
  if (!char.value) return;
  import("../store").then((m) => {
    store.aiPanelOpen = true;
    m.startAi("人物卡", prompts.buildCharacterCard(char.value.name, char.value.role));
  });
}

// ---------- 关系网画布 ----------
const canvas = ref(null);
const cw = 1800;
const ch = 1200;
const drag = ref(null);

function pos(id) {
  if (!store.canvasPos[id]) {
    const chars = store.novel?.characters || [];
    const i = chars.findIndex((c) => c.id === id);
    const angle = (Math.PI * 2 * i) / Math.max(1, chars.length);
    store.canvasPos[id] = { x: cw / 2 + 400 * Math.cos(angle), y: ch / 2 + 300 * Math.sin(angle) };
  }
  return store.canvasPos[id];
}
function startDrag(id, e) {
  drag.value = { id, dx: e.clientX - pos(id).x, dy: e.clientY - pos(id).y };
}
function onMove(e) {
  if (!drag.value) return;
  const p = store.canvasPos[drag.value.id];
  p.x = Math.max(0, Math.min(cw - 140, e.clientX - drag.value.dx));
  p.y = Math.max(0, Math.min(ch - 50, e.clientY - drag.value.dy));
}
function onUp() {
  drag.value = null;
}
function resetCanvas() {
  store.canvasPos = {};
}
const edges = computed(() => {
  const out = [];
  for (const c of store.novel?.characters || []) {
    for (const r of c.relationships || []) {
      const b = store.novel.characters.find((x) => x.id === r.target_id);
      if (b) out.push({ a: pos(c.id), b: pos(b.id), relation: r.relation });
    }
  }
  return out;
});

onMounted(() => {
  window.addEventListener("mousemove", onMove);
  window.addEventListener("mouseup", onUp);
});
onBeforeUnmount(() => {
  window.removeEventListener("mousemove", onMove);
  window.removeEventListener("mouseup", onUp);
});

// ---------- 任务看板 ----------
const cols = [
  { name: "待办", color: "var(--text-2)" },
  { name: "进行中", color: "var(--warn)" },
  { name: "已完成", color: "var(--ok)" },
];
const filteredTasks = computed(() => {
  const sel = store.selChain;
  return (store.novel?.tasks || []).filter((t) => sel === "all" || t.chain_id === sel);
});
function colTasks(ci) {
  return filteredTasks.value.filter((t) => t.status === ci);
}
function newTask() {
  const t = { id: "t" + Math.random().toString(36).slice(2, 8), title: "新任务", description: "", status: 0, chain_id: store.selChain === "all" ? null : store.selChain };
  store.novel.tasks.push(t);
  save();
}
function moveTask(t, status) {
  t.status = status;
  save();
}
function cycle(t) {
  t.status = (t.status + 1) % 3;
  save();
}
function openTaskMenu(e, t) {
  window.dispatchEvent(
    new CustomEvent("jinshu:contextmenu", {
      detail: {
        x: e.clientX,
        y: e.clientY,
        items: [
          {
            label: "编辑标题",
            run: () => {
              const n = prompt("任务标题", t.title);
              if (n) {
                t.title = n;
                save();
              }
            },
          },
          { label: "删除任务", danger: true, run: () => { store.novel.tasks = store.novel.tasks.filter((x) => x.id !== t.id); save(); } },
        ],
      },
    })
  );
}

// ---------- 大纲详情 ----------
const outNode = computed(() => {
  const find = (arr) => {
    for (const n of arr) {
      if (n.id === store.selOutline) return n;
      const f = find(n.children || []);
      if (f) return f;
    }
    return null;
  };
  return store.novel ? find(store.novel.outline) : null;
});
</script>

<style scoped>
.detail { flex: 1; min-height: 0; display: flex; flex-direction: column; }
.d-body { flex: 1; overflow-y: auto; padding: 16px 22px; display: flex; flex-direction: column; gap: 8px; max-width: 860px; }
.d-head { display: flex; align-items: center; gap: 10px; }
.d-title { font-size: 16px; font-weight: 700; }
.d-hint { font-size: 11px; color: var(--text-3); }
.d-actions { margin-left: auto; display: flex; gap: 6px; }
.row2 { display: flex; align-items: center; gap: 8px; }
.row2 label, .field label { font-size: 12.5px; color: var(--text-2); width: 90px; flex-shrink: 0; }
.field { display: flex; flex-direction: column; gap: 4px; }
.field label { width: auto; margin-bottom: 2px; }
.row2 .input { flex: 1; max-width: 420px; }

.rel-row { display: flex; align-items: center; gap: 10px; padding: 3px 0; }
.rel-name { font-size: 12.5px; color: var(--accent); }
.rel-text { font-size: 12px; color: var(--text-2); }
.rel-add { display: flex; gap: 6px; margin-top: 4px; }

.canvas-wrap { flex: 1; display: flex; flex-direction: column; min-height: 0; padding: 14px 22px; }
.canvas { flex: 1; position: relative; overflow: auto; background: var(--panel); border: 1px solid var(--border); border-radius: 10px; }
.canvas-svg { position: absolute; inset: 0; }
.cnode {
  position: absolute;
  padding: 10px 20px;
  border-radius: 10px;
  border: 1px solid var(--accent);
  background: var(--panel-alt);
  color: var(--text);
  font-size: 13px;
  cursor: grab;
  user-select: none;
  box-shadow: var(--shadow-sm);
  white-space: nowrap;
}
.cnode:hover { background: var(--hover); }
.cnode.sel { background: var(--accent-soft); border-color: var(--accent-strong); }
.cnode:active { cursor: grabbing; }

.board { flex: 1; display: flex; gap: 16px; padding: 14px 22px 20px; overflow-x: auto; }
.bcol {
  width: 230px;
  min-width: 200px;
  flex-shrink: 0;
  border-radius: 10px;
  border: 1px solid var(--border);
  background: var(--panel);
  display: flex;
  flex-direction: column;
  max-height: 100%;
}
.bcol-head { padding: 10px 14px; font-size: 13px; font-weight: 600; color: var(--colc); border-bottom: 1px solid var(--border); }
.bcol-body { flex: 1; overflow-y: auto; padding: 8px; display: flex; flex-direction: column; gap: 8px; }
.task-card {
  border-radius: 8px;
  border: 1px solid var(--border);
  background: var(--panel-alt);
  padding: 9px 11px;
  cursor: pointer;
}
.task-card:hover { border-color: var(--border-strong); }
.tc-title { font-size: 12.5px; color: var(--text); }
.tc-desc { font-size: 10.5px; color: var(--text-3); margin-top: 3px; }
.tc-ops { display: flex; gap: 4px; margin-top: 6px; }
.tco { border: none; background: transparent; color: var(--text-3); cursor: pointer; font-size: 10px; padding: 2px 5px; border-radius: 4px; }
.tco:hover { background: var(--hover); color: var(--text); }
</style>
