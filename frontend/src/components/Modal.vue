<template>
  <div class="overlay" @mousedown.self="close">
    <div class="popup modal slide-up" :style="{ width: width + 'px' }" @keydown.stop>
      <div class="modal-head">
        <span class="modal-title">{{ title }}</span>
        <button class="icon-btn" @click="close">✕</button>
      </div>
      <div class="modal-body">
        <!-- 新建小说 -->
        <template v-if="d.kind === 'newNovel'">
          <label>书名</label>
          <input class="input" v-model="form.title" placeholder="例如：剑出昆仑" autofocus @keydown.enter="doCreate" />
          <label>作者</label>
          <input class="input" v-model="form.author" placeholder="你的笔名" />
          <label>题材</label>
          <input class="input" v-model="form.genre" placeholder="例如：仙侠 / 都市 / 玄幻 / 悬疑" />
          <label>简介 / 核心设定</label>
          <textarea class="input" rows="4" v-model="form.desc" placeholder="一句话讲清这本书要写什么……"></textarea>
          <div class="modal-foot">
            <button class="btn" @click="close">取消</button>
            <button class="btn primary" :disabled="!form.title.trim()" @click="doCreate">创建</button>
          </div>
        </template>

        <!-- 新建章节 -->
        <template v-else-if="d.kind === 'newChapter'">
          <label>章节标题</label>
          <input class="input" v-model="form.title" placeholder="例如：第一章 少年出山" autofocus @keydown.enter="doNewChapter" />
          <label>所属卷</label>
          <select class="input" v-model="form.volId">
            <option v-for="v in volumes" :key="v.id" :value="v.id">{{ v.title }}</option>
            <option value="">（自动创建「正文」卷）</option>
          </select>
          <div class="modal-foot">
            <button class="btn" @click="close">取消</button>
            <button class="btn primary" :disabled="!form.title.trim()" @click="doNewChapter">创建并打开</button>
          </div>
        </template>

        <!-- 新建卷 -->
        <template v-else-if="d.kind === 'newVolume'">
          <label>卷名</label>
          <input class="input" v-model="form.title" placeholder="例如：第一卷 风起青萍" autofocus @keydown.enter="doNewVolume" />
          <div class="modal-foot">
            <button class="btn" @click="close">取消</button>
            <button class="btn primary" :disabled="!form.title.trim()" @click="doNewVolume">创建</button>
          </div>
        </template>

        <!-- 重命名 -->
        <template v-else-if="d.kind === 'renameChapter' || d.kind === 'renameVolume'">
          <label>新名称</label>
          <input class="input" v-model="form.title" autofocus @keydown.enter="doRename" />
          <div class="modal-foot">
            <button class="btn" @click="close">取消</button>
            <button class="btn primary" :disabled="!form.title.trim()" @click="doRename">确定</button>
          </div>
        </template>

        <!-- 删除确认 -->
        <template v-else-if="d.kind === 'deleteChapter' || d.kind === 'deleteVolume' || d.kind === 'deleteNovel'">
          <p class="confirm-text">{{ confirmText }}</p>
          <div class="modal-foot">
            <button class="btn" @click="close">取消</button>
            <button class="btn danger" @click="doDelete">删除</button>
          </div>
        </template>

        <!-- 导出 -->
        <template v-else-if="d.kind === 'export'">
          <label>格式</label>
          <div class="fmt-row">
            <button v-for="f in fmts" :key="f.id" class="pill" :class="{ active: form.fmt === f.id }" @click="form.fmt = f.id">
              {{ f.label }}
            </button>
          </div>
          <p class="hint">{{ fmtHint }}</p>
          <template v-if="form.fmt === 'jsb'">
            <label>设置密码（Scrypt 派生密钥，密码不落盘）</label>
            <input class="input" type="password" v-model="form.pwd" placeholder="密码" />
            <input class="input" type="password" v-model="form.pwd2" placeholder="再次输入密码" style="margin-top:6px" />
            <p class="hint err" v-if="form.pwd && form.pwd !== form.pwd2">两次密码不一致</p>
          </template>
          <div class="modal-foot">
            <button class="btn" @click="close">取消</button>
            <button class="btn primary" :disabled="!canExport" @click="doExport">导出</button>
          </div>
        </template>

        <!-- 导入 -->
        <template v-else-if="d.kind === 'import'">
          <label>备份文件密码</label>
          <input class="input" type="password" v-model="form.pwd" placeholder="备份文件的密码" autofocus />
          <p class="hint">选择 .jsb 加密备份文件，验证密码后导入为一部新小说。</p>
          <div class="modal-foot">
            <button class="btn" @click="close">取消</button>
            <button class="btn primary" :disabled="!form.pwd" @click="doImport">选择文件并导入</button>
          </div>
        </template>

        <!-- 本地起名机 -->
        <template v-else-if="d.kind === 'namer'">
          <div class="namer-opts">
            <label>类型</label>
            <div class="fmt-row">
              <button v-for="t in namerTypes" :key="t.id" class="pill" :class="{ active: namer.type === t.id }" @click="namer.type = t.id">{{ t.label }}</button>
            </div>
            <label>风格</label>
            <div class="fmt-row">
              <button v-for="st in namerStyles" :key="st.id" class="pill" :class="{ active: namer.style === st.id }" @click="namer.style = st.id">{{ st.label }}</button>
            </div>
            <label v-if="namer.type === 'person'">性别倾向</label>
            <div class="fmt-row" v-if="namer.type === 'person'">
              <button v-for="g in namerGenders" :key="g" class="pill" :class="{ active: namer.gender === g }" @click="namer.gender = g">{{ g }}</button>
            </div>
            <div class="namer-actions">
              <button class="btn primary" @click="doGenerate">🎲 生成 {{ namer.count }} 个</button>
              <button class="btn" @click="namer.count = namer.count === 8 ? 16 : 8">{{ namer.count === 8 ? "更多（16）" : "精简（8）" }}</button>
            </div>
          </div>
          <div class="namer-list">
            <div v-for="(item, i) in namerResults" :key="i" class="namer-item">
              <div class="ni-name">{{ item.name }}</div>
              <div class="ni-mean">{{ item.meaning }}</div>
              <div class="ni-ops">
                <button class="btn sm" @click="copyName(item.name)">复制</button>
                <button class="btn sm" @click="insertName(item.name)">插入到正文</button>
              </div>
            </div>
            <div v-if="!namerResults.length" class="hint" style="text-align:center;padding:14px 0">
              点击「生成」获取本地算法起的名字（不消耗 AI 额度）
            </div>
          </div>
        </template>

        <!-- 关于 -->
        <template v-else-if="d.kind === 'about'">
          <div class="about">
            <div class="about-logo">📖</div>
            <div class="about-title">锦书 · 小说编辑器 v0.1.0</div>
            <p>为中文小说创作而生的本地编辑器（Tauri + Vue 3）。</p>
            <p>
              · 数据 AES-256-GCM 加密存储于安装目录，不写入系统目录<br />
              · 支持 txt / md 导出与 .jsb 密码加密备份<br />
              · 接入任意 OpenAI 兼容 / Anthropic API：大纲、续写、润色、评审一站式<br />
              · 章节树 / 大纲 / 人物关系网 / 世界观 / 时间线 / 任务看板 / 写作统计
            </p>
            <p class="dim">技术栈：Rust + Tauri 2 · Vue 3 · CodeMirror 6 · 开源协议 MIT</p>
          </div>
          <div class="modal-foot">
            <button class="btn primary" @click="close">好的</button>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<script setup>
import { reactive, computed, ref } from "vue";
import { store, toast, createNovel, openNovel, saveAll , allChapters } from "../store";
import { api } from "../api";
import { save, open } from "@tauri-apps/plugin-dialog";
import { generateNames, STYLES, GENDERS } from "../names";

const d = computed(() => store.dialog);
const form = reactive({
  title: "",
  author: "",
  genre: "",
  desc: "",
  volId: "",
  fmt: "md",
  pwd: "",
  pwd2: "",
});

const volumes = computed(() => store.novel?.volumes || []);

const title = computed(() => {
  switch (d.value?.kind) {
    case "newNovel": return "📝 新建小说";
    case "newChapter": return "📄 新建章节";
    case "newVolume": return "📁 新建分卷";
    case "renameChapter": return "重命名章节";
    case "renameVolume": return "重命名卷";
    case "deleteChapter": return "🗑 删除章节";
    case "deleteVolume": return "🗑 删除卷";
    case "deleteNovel": return "🗑 删除小说";
    case "export": return "📤 导出作品";
    case "import": return "📥 导入 .jsb 备份";
    case "namer": return "🎲 本地起名机（不消耗 AI）";
    case "about": return "关于锦书";
    default: return "";
  }
});

const confirmText = computed(() => {
  const p = d.value?.payload || {};
  switch (d.value?.kind) {
    case "deleteChapter": return `确定删除章节《${p.title}》吗？此操作不可恢复。`;
    case "deleteVolume": return `确定删除卷《${p.title}》及其全部章节吗？此操作不可恢复。`;
    case "deleteNovel": return `确定删除《${p.title}》吗？全部章节数据将被清除，且不可恢复。`;
    default: return "";
  }
});

const fmts = [
  { id: "txt", label: "纯文本 (.txt)" },
  { id: "md", label: "Markdown (.md)" },
  { id: "jsb", label: "加密备份 (.jsb)" },
];
const fmtHint = computed(() => ({
  txt: "通用纯文本，可导入任何写作平台",
  md: "保留卷/章标题结构",
  jsb: "密码保护，可跨设备恢复",
}[form.fmt]));

const canExport = computed(() => form.fmt !== "jsb" || (form.pwd && form.pwd === form.pwd2));

function close() {
  store.dialog = null;
}

async function doCreate() {
  const ok = await createNovel({
    title: form.title,
    author: form.author,
    genre: form.genre,
    description: form.desc,
  });
  if (ok) {
    Object.assign(form, { title: "", author: "", genre: "", desc: "" });
    close();
  }
}

async function doNewChapter() {
  const n = store.novel;
  let volId = form.volId;
  if (!volId) {
    n.add_volume("正文");
    volId = n.volumes[n.volumes.length - 1].id;
  }
  const cid = n.add_chapter(volId, form.title.trim());
  store.chapters[cid] = "";
  store.dirty[cid] = true;
  await api.saveNovel(n);
  const { openTab } = await import("../store");
  openTab(cid);
  form.title = "";
  close();
  toast("章节已创建");
}

async function doNewVolume() {
  store.novel.add_volume(form.title.trim());
  await api.saveNovel(store.novel);
  form.title = "";
  close();
  toast("卷已创建");
}

async function doRename() {
  const p = d.value?.payload;
  const name = form.title.trim();
  if (d.value.kind === "renameChapter") {
    const c = allChapters(store.novel).find((x) => x.id === p.cid);
    if (c) c.title = name;
  } else {
    const v = store.novel.volumes.find((x) => x.id === p.vid);
    if (v) v.title = name;
  }
  await api.saveNovel(store.novel);
  form.title = "";
  close();
}

async function doDelete() {
  const p = d.value?.payload;
  const n = store.novel;
  try {
    if (d.value.kind === "deleteChapter") {
      await api.deleteChapter(n.meta.id, p.cid);
      delete store.chapters[p.cid];
      delete store.dirty[p.cid];
      store.openTabs = store.openTabs.filter((c) => c !== p.cid);
      if (store.activeTab === p.cid) store.activeTab = store.openTabs[store.openTabs.length - 1] || null;
      toast("章节已删除");
    } else if (d.value.kind === "deleteVolume") {
      const removed = n.volumes.find((v) => v.id === p.vid)?.chapters || [];
      for (const c of removed) await api.deleteChapter(n.meta.id, c.id);
      n.volumes = n.volumes.filter((v) => v.id !== p.vid);
      await api.saveNovel(n);
      toast("卷已删除");
    } else if (d.value.kind === "deleteNovel") {
      await api.deleteNovel(p.id);
      if (n?.meta?.id === p.id) {
        store.novel = null;
        store.chapters = {};
        store.openTabs = [];
        store.activeTab = null;
      }
      store.library = await api.listNovels();
      toast("已删除");
    }
    close();
  } catch (e) {
    toast(String(e), false);
  }
}

async function doExport() {
  const n = store.novel;
  const chapters = allChapters(n).map((c) => [c.id, c.title, store.chapters[c.id] || ""]);
  const ext = form.fmt;
  const defaultName = (n.meta.title || "未命名").replace(/[\\/:*?"<>|]/g, "_") + "." + ext;
  let path = null;
  try {
    path = await save({
      title: "导出作品",
      defaultPath: defaultName,
      filters: [{ name: ext.toUpperCase(), extensions: [ext] }],
    });
  } catch (e) {
    /* 对话框取消 */
  }
  if (!path) {
    close();
    return;
  }
  try {
    await api.exportWork({
      fmt: form.fmt,
      path,
      password: form.fmt === "jsb" ? form.pwd : null,
      novel: n,
      chapters,
    });
    toast(`已导出：${path}`);
  } catch (e) {
    toast(`导出失败：${e}`, false);
  }
  close();
}

async function doImport() {
  let path = null;
  try {
    path = await open({ title: "选择 .jsb 备份", filters: [{ name: "JSB 备份", extensions: ["jsb"] }] });
  } catch (e) {
    /* cancelled */
  }
  if (!path) return;
  try {
    const id = await api.importJsb(path, form.pwd);
    store.library = await api.listNovels();
    close();
    await openNovel(id);
    toast("备份已导入");
  } catch (e) {
    toast(`导入失败：${e}`, false);
  }
}

const width = computed(() =>
  d.value?.kind === "newNovel" ? 480
  : d.value?.kind === "namer" ? 560
  : d.value?.kind === "about" ? 440 : 420
);

// ---------- 本地起名机 ----------
const namerTypes = [
  { id: "person", label: "人物名" },
  { id: "book", label: "书名" },
  { id: "place", label: "地名" },
];
const namerStyles = STYLES;
const namerGenders = GENDERS;
const namer = reactive({
  type: "person",
  style: "gufeng",
  gender: "中性",
  count: 8,
});
const namerResults = ref([]);

function existingNames() {
  const n = store.novel;
  if (!n) return [];
  if (namer.type === "person") return (n.characters || []).map((c) => c.name);
  if (namer.type === "book") return n.meta.title ? [n.meta.title] : [];
  return (n.locations || []).map((l) => l.name);
}
function doGenerate() {
  namerResults.value = generateNames({
    type: namer.type,
    style: namer.style,
    gender: namer.gender,
    count: namer.count,
    exclude: existingNames(),
  });
}
function copyName(name) {
  navigator.clipboard.writeText(name);
  toast("已复制：" + name);
}
function insertName(name) {
  const cid = store.activeTab;
  if (!cid) {
    toast("请先打开一个章节", false);
    return;
  }
  window.dispatchEvent(new CustomEvent("jinshu:insert", { detail: { cid, content: name } }));
  toast("已插入到正文");
}
</script>

<style scoped>
.modal { width: 420px; max-width: 92vw; overflow: hidden; }
.modal-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 18px 0;
}
.modal-title { font-size: 15.5px; font-weight: 600; }
.modal-body { padding: 14px 18px 18px; display: flex; flex-direction: column; gap: 7px; }
label { font-size: 12.5px; color: var(--text-2); margin-top: 4px; }
.modal-foot { display: flex; justify-content: flex-end; gap: 8px; margin-top: 16px; }
.fmt-row { display: flex; gap: 6px; flex-wrap: wrap; }
.hint { font-size: 11.5px; color: var(--text-3); }
.hint.err { color: var(--danger); }
.confirm-text { font-size: 13px; line-height: 1.7; padding: 4px 0; }
.about { text-align: center; padding: 6px 0; }
.about-logo { font-size: 38px; }
.about-title { font-size: 15px; font-weight: 600; margin: 6px 0 10px; }
.about p { font-size: 12.5px; color: var(--text-2); line-height: 1.9; text-align: left; margin-bottom: 6px; }
.about .dim { color: var(--text-3); font-size: 11.5px; }
</style>
