<template>
  <div class="settings">
    <div class="st-head">⚙️ 设置</div>
    <div class="st-body">
      <!-- 外观 -->
      <div class="section">
        <div class="sec-title">外观</div>
        <div class="sec-body">
          <div class="row">
            <span class="row-label">主题</span>
            <button class="pill" :class="{ active: store.settings.theme !== 'light' }" @click="setTheme('dark')">深色</button>
            <button class="pill" :class="{ active: store.settings.theme === 'light' }" @click="setTheme('light')">浅色</button>
          </div>
          <div class="row">
            <span class="row-label">强调色</span>
            <span
              v-for="(c, i) in accents"
              :key="i"
              class="swatch"
              :style="{ background: `rgb(${c[1][0]}, ${c[1][1]}, ${c[1][2]})` }"
              :class="{ sel: isAccent(c[1]) }"
              :title="c[0]"
              @click="setAccent(c[1])"
            ></span>
          </div>
          <div class="row">
            <span class="row-label">界面缩放</span>
            <input type="range" min="0.8" max="1.6" step="0.05" v-model.number="store.settings.ui_scale" @change="persist" />
            <span class="row-val">{{ store.settings.ui_scale }}x</span>
          </div>
        </div>
      </div>

      <!-- 编辑 -->
      <div class="section">
        <div class="sec-title">编辑</div>
        <div class="sec-body">
          <div class="row">
            <span class="row-label">正文字体</span>
            <button class="pill" :class="{ active: store.settings.editor.font === 'serif' }" @click="setFont('serif')">宋体类（衬线）</button>
            <button class="pill" :class="{ active: store.settings.editor.font !== 'serif' }" @click="setFont('sans')">黑体类（无衬线）</button>
          </div>
          <div class="row">
            <span class="row-label">字号</span>
            <input type="range" min="12" max="32" step="1" v-model.number="store.settings.editor.font_size" @change="persist" />
            <span class="row-val">{{ store.settings.editor.font_size }}px</span>
          </div>
          <div class="row">
            <span class="row-label">行距</span>
            <input type="range" min="1" max="2.8" step="0.05" v-model.number="store.settings.editor.line_spacing" @change="persist" />
            <span class="row-val">{{ store.settings.editor.line_spacing }}x</span>
          </div>
          <div class="row">
            <span class="row-label">自动换行</span>
            <label class="switch"><input type="checkbox" v-model="store.settings.editor.wrap" @change="persist" /><span class="slider"></span></label>
          </div>
          <div class="row">
            <span class="row-label">行号</span>
            <label class="switch"><input type="checkbox" v-model="store.settings.editor.show_line_numbers" @change="persist" /><span class="slider"></span></label>
          </div>
          <div class="row">
            <span class="row-label">Markdown 高亮</span>
            <label class="switch"><input type="checkbox" v-model="store.settings.editor.markdown_highlight" @change="persist" /><span class="slider"></span></label>
            <span class="hint">中文写作建议关闭，# * 符号原样显示</span>
          </div>
          <div class="row">
            <span class="row-label">自动保存间隔</span>
            <input type="range" min="1" max="120" step="1" v-model.number="store.settings.autosave_secs" @change="persist" />
            <span class="row-val">{{ store.settings.autosave_secs }}秒</span>
          </div>
        </div>
      </div>

      <!-- AI 服务 -->
      <div class="section">
        <div class="sec-title">AI 服务 <span class="hint">（API Key 加密存储于本地，仅在你调用时发送给你配置的服务商）</span></div>
        <div class="sec-body">
          <div class="row">
            <span class="row-label">协议</span>
            <button class="pill" :class="{ active: store.settings.ai.protocol !== 'anthropic' }" @click="setProtocol('openai')">OpenAI 兼容</button>
            <button class="pill" :class="{ active: store.settings.ai.protocol === 'anthropic' }" @click="setProtocol('anthropic')">Anthropic</button>
          </div>
          <div class="row">
            <span class="row-label">服务商预设</span>
            <button v-for="p in providers" :key="p.name" class="pill" @click="applyProvider(p)">{{ p.name }}</button>
          </div>
          <div class="row">
            <span class="row-label">Base URL</span>
            <input class="input" style="flex:1;max-width:420px" v-model="store.settings.ai.base_url" @change="persist" placeholder="https://…/v1" />
          </div>
          <div class="row">
            <span class="row-label">模型</span>
            <input class="input" style="flex:1;max-width:420px" v-model="store.settings.ai.model" @change="persist" placeholder="模型名称" />
          </div>
          <div class="row">
            <span class="row-label">API Key</span>
            <input
              v-if="showKey"
              class="input"
              style="flex:1;max-width:420px"
              v-model="store.settings.ai.api_key"
              @change="persist"
              placeholder="sk-…"
            />
            <span v-else class="masked">{{ maskedKey }}</span>
            <button class="btn sm" @click="showKey = !showKey">{{ showKey ? "🙈 隐藏" : "👁 显示" }}</button>
            <button class="btn sm" :disabled="testing" @click="testAI">{{ testing ? "测试中…" : "测试连接" }}</button>
          </div>
          <div class="row">
            <span class="row-label">温度</span>
            <input type="range" min="0" max="1.5" step="0.05" v-model.number="store.settings.ai.temperature" @change="persist" />
            <span class="row-val">{{ store.settings.ai.temperature }}</span>
          </div>
          <div class="row">
            <span class="row-label">最大 Token</span>
            <input class="input" style="width:110px" type="number" min="256" max="32000" v-model.number="store.settings.ai.max_tokens" @change="persist" />
            <span class="row-label" style="margin-left:16px">超时（秒）</span>
            <input class="input" style="width:110px" type="number" min="30" max="900" v-model.number="store.settings.ai.timeout_secs" @change="persist" />
          </div>
          <div class="row">
            <span class="row-label">自动注入设定</span>
            <label class="switch"><input type="checkbox" v-model="store.settings.ai.inject_lore" @change="persist" /><span class="slider"></span></label>
            <span class="hint">正文出现人物/地名时自动附带对应卡片</span>
          </div>
          <div class="row col">
            <span class="row-label">系统提示词</span>
            <textarea class="input" rows="3" v-model="store.settings.ai.system_prompt" @change="persist"></textarea>
          </div>
          <div class="ai-test-result ok" v-if="store.aiTestResult && store.aiTestResult[0]">✅ 连接成功：{{ store.aiTestResult[1] }}</div>
          <div class="ai-test-result err" v-else-if="store.aiTestResult">❌ 连接失败：{{ store.aiTestResult[1] }}</div>
        </div>
      </div>

      <!-- 存储与安全 -->
      <div class="section">
        <div class="sec-title">存储与安全</div>
        <div class="sec-body">
          <p class="info">数据目录：<code>{{ store.dataDir }}</code></p>
          <p class="info dim">
            所有作品、章节与设置均以 AES-256-GCM 加密文件存储于此目录，密钥为本机随机生成的 32 字节文件，不写入任何系统目录。
          </p>
          <p class="info ok">密钥指纹：{{ store.keyFp }}</p>
          <div class="row" style="margin-top:8px">
            <button class="btn" @click="openDataDir">📂 打开数据目录</button>
            <button class="btn" @click="store.dialog = { kind: 'export' }">📤 导出 .jsb 加密备份</button>
            <button class="btn" @click="store.dialog = { kind: 'import' }">📥 导入 .jsb 备份</button>
          </div>
          <p class="info dim">提示：.jsb 备份使用你设定的密码（Scrypt 派生密钥），密码不落盘，可跨设备恢复；请妥善保管密码。</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from "vue";
import { store, saveSettings, applyTheme, toast } from "../store";
import { api } from "../api";

const showKey = ref(false);
const testing = ref(false);

const accents = [
  ["晨曦蓝", [76, 141, 255]],
  ["星云紫", [168, 130, 255]],
  ["松石青", [63, 200, 200]],
  ["鎏金橙", [233, 151, 63]],
  ["竹叶绿", [61, 190, 110]],
  ["绯红", [250, 110, 156]],
  ["黛紫蓝", [110, 139, 255]],
  ["樱粉", [242, 139, 194]],
];

const providers = [
  { name: "DeepSeek", protocol: "openai", base_url: "https://api.deepseek.com/v1", model: "deepseek-chat" },
  { name: "Moonshot", protocol: "openai", base_url: "https://api.moonshot.cn/v1", model: "moonshot-v1-8k" },
  { name: "通义千问", protocol: "openai", base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1", model: "qwen-plus" },
  { name: "智谱 GLM", protocol: "openai", base_url: "https://open.bigmodel.cn/api/paas/v4", model: "glm-4-flash" },
  { name: "Ollama 本地", protocol: "openai", base_url: "http://localhost:11434/v1", model: "qwen2.5" },
];

const maskedKey = computed(() => {
  const k = store.settings.ai.api_key;
  if (!k) return "（未填写）";
  return "••••••••••••" + k.slice(-4);
});

function persist() {
  saveSettings();
  applyTheme();
}
function setTheme(t) {
  store.settings.theme = t;
  persist();
}
function isAccent(rgb) {
  const a = store.settings.accent || [110, 139, 255];
  return a[0] === rgb[0] && a[1] === rgb[1] && a[2] === rgb[2];
}
function setAccent(rgb) {
  store.settings.accent = rgb;
  persist();
}
function setFont(f) {
  store.settings.editor.font = f;
  persist();
}
function setProtocol(p) {
  store.settings.ai.protocol = p;
  if (p === "anthropic" && !store.settings.ai.base_url.includes("anthropic")) {
    store.settings.ai.base_url = "https://api.anthropic.com";
    store.settings.ai.model = "claude-sonnet-4-5";
  }
  persist();
}
function applyProvider(p) {
  store.settings.ai.protocol = p.protocol;
  store.settings.ai.base_url = p.base_url;
  store.settings.ai.model = p.model;
  persist();
}
async function testAI() {
  testing.value = true;
  try {
    await api.aiTest(store.settings.ai);
    setTimeout(() => (testing.value = false), 1500);
  } catch (e) {
    testing.value = false;
    toast(String(e), false);
  }
}
function openDataDir() {
  api.openDir(store.dataDir);
}
</script>

<style scoped>
.settings { flex: 1; overflow-y: auto; padding: 18px 22px 40px; }
.st-head { font-size: 19px; font-weight: 700; margin-bottom: 16px; }
.st-body { display: flex; flex-direction: column; gap: 14px; max-width: 1100px; }
.section { border: 1px solid var(--border); border-radius: 10px; background: var(--panel); overflow: hidden; }
.sec-title { font-size: 13.5px; font-weight: 600; color: var(--accent); padding: 11px 16px; border-bottom: 1px solid var(--border); background: var(--panel-alt); }
.sec-body { padding: 12px 16px; display: flex; flex-direction: column; gap: 9px; }
.row { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
.row.col { flex-direction: column; align-items: stretch; }
.row-label { font-size: 12.5px; color: var(--text-2); width: 108px; flex-shrink: 0; }
.row-val { font-size: 12px; color: var(--text-2); min-width: 34px; }
.hint { font-size: 10.5px; color: var(--text-3); }
.swatch { width: 24px; height: 24px; border-radius: 7px; cursor: pointer; border: 2px solid transparent; }
.swatch.sel { border-color: var(--text); }
.swatch:hover { transform: scale(1.1); }
input[type="range"] { accent-color: var(--accent); width: 200px; }
.masked { font-size: 13px; color: var(--text-2); letter-spacing: 2px; }
.info { font-size: 12px; color: var(--text-2); line-height: 1.8; word-break: break-all; }
.info code { background: var(--panel-alt); padding: 2px 6px; border-radius: 4px; font-size: 11.5px; }
.info.dim { color: var(--text-3); }
.info.ok { color: var(--ok); }
.ai-test-result { font-size: 12px; padding: 8px 12px; border-radius: 6px; }
.ai-test-result.ok { background: var(--ok-soft); color: var(--ok); }
.ai-test-result.err { background: var(--danger-soft); color: var(--danger); }

.switch { position: relative; display: inline-block; width: 36px; height: 20px; flex-shrink: 0; }
.switch input { opacity: 0; width: 0; height: 0; }
.slider {
  position: absolute; inset: 0; cursor: pointer;
  background: var(--selected); border-radius: 10px; transition: 0.15s;
}
.slider::before {
  content: ""; position: absolute; width: 14px; height: 14px;
  left: 3px; top: 3px; border-radius: 50%; background: var(--text-2); transition: 0.15s;
}
.switch input:checked + .slider { background: var(--accent); }
.switch input:checked + .slider::before { transform: translateX(16px); background: #fff; }
</style>
