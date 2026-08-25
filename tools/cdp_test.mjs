// 锦书 CDP 端到端测试：驱动 WebView2 完成 新建小说 → 写正文 → 自动保存 全流程
// 用法: node tools/cdp_test.mjs [步骤参数]
import { writeFileSync } from "node:fs";

const list = await fetch("http://localhost:9222/json").then((r) => r.json());
const page = list.find((p) => p.type === "page");
if (!page) {
  console.error("未找到页面，请先启动应用（WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222）");
  process.exit(1);
}

const ws = new WebSocket(page.webSocketDebuggerUrl);
await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej; });

let msgId = 0;
const pending = new Map();
ws.onmessage = (e) => {
  const m = JSON.parse(e.data);
  if (m.id && pending.has(m.id)) {
    pending.get(m.id)(m);
    pending.delete(m.id);
  }
};
function send(method, params = {}) {
  return new Promise((res) => {
    const i = ++msgId;
    pending.set(i, res);
    ws.send(JSON.stringify({ id: i, method, params }));
  });
}
async function js(expr) {
  const r = await send("Runtime.evaluate", { expression: expr, awaitPromise: true, returnByValue: true });
  if (r.result?.exceptionDetails) throw new Error("JS错误: " + JSON.stringify(r.result.exceptionDetails));
  return r.result?.result?.value;
}
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
async function waitFor(expr, timeout = 8000) {
  const t0 = Date.now();
  while (Date.now() - t0 < timeout) {
    if (await js(expr)) return true;
    await sleep(200);
  }
  return false;
}
async function shot(name) {
  const r = await send("Page.captureScreenshot", { format: "png" });
  writeFileSync(`C:/Code/JinShu-rust/${name}.png`, Buffer.from(r.result.data, "base64"));
  console.log(`📸 ${name}.png`);
}

const step = process.argv[2] || "all";
console.log("== 锦书 CDP 测试 ==");

if (step === "all" || step === "create") {
  console.log("1️⃣ 点击「新建小说」卡片");
  await js(`document.querySelectorAll('.w-card')[0]?.click()`);
  await sleep(500);
  const modalOpen = await js(`!!document.querySelector('.modal')`);
  console.log("   弹窗打开:", modalOpen);
  if (!modalOpen) throw new Error("弹窗未打开");

  console.log("2️⃣ 填写表单并创建");
  await js(`(() => {
    const inputs = document.querySelectorAll('.modal input');
    const set = (el, v) => { const s = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set; s.call(el, v); el.dispatchEvent(new Event('input', { bubbles: true })); };
    set(inputs[0], '剑出昆仑');
    set(inputs[1], '云中客');
    set(inputs[2], '仙侠');
    const ta = document.querySelector('.modal textarea');
    const s2 = Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, 'value').set;
    s2.call(ta, '少年持剑下山，踏入仙门纷争。');
    ta.dispatchEvent(new Event('input', { bubbles: true }));
    return true;
  })()`);
  await sleep(300);
  await js(`[...document.querySelectorAll('.modal .btn')].find(b => b.textContent.includes('创建'))?.click()`);
  console.log("   等待编辑器出现…");
  const ok = await waitFor(`!!document.querySelector('.cm-editor')`);
  console.log("   编辑器出现:", ok);
  if (!ok) throw new Error("编辑器未出现");
  await sleep(1500);
  await shot("t_cdp_editor");
}

if (step === "all" || step === "type") {
  console.log("3️⃣ 在编辑器中输入正文");
  await js(`document.querySelector('.cm-content')?.focus()`);
  await sleep(300);
  await send("Input.insertText", {
    text: "　　天光未亮，山道上的霜已经结了一层。\n\n少年负剑而行，衣角被晨风卷起。\n\n“师父说，剑出昆仑，天下皆可去得。”他低声自语，握紧了剑柄。",
  });
  await sleep(500);
  const words = await js(`document.querySelector('.ch-words')?.textContent`);
  console.log("   字数显示:", words);
  console.log("4️⃣ 等待自动保存（2 秒防抖）…");
  await sleep(3500);
  const dirty = await js(`Object.keys(window.__vue_dirty || {}).length`);
  console.log("   保存状态: 等文件验证");
  await shot("t_cdp_typed");
}

if (step === "all" || step === "verify") {
  console.log("5️⃣ 验证后端加密落盘");
  const { execSync } = await import("node:child_process");
  const out = execSync('ls C:/Code/JinShu-rust/src-tauri/target/debug/data/novels/ 2>/dev/null').toString().trim();
  console.log("   novels:", out || "（空！）");
  if (!out) throw new Error("小说未落盘");
  const files = execSync('find C:/Code/JinShu-rust/src-tauri/target/debug/data/novels/ -type f 2>/dev/null').toString().trim().split("\n");
  console.log("   加密文件:", files.map((f) => f.split("/").pop()).join(", "));
  const leak = execSync(`grep -rl "剑出昆仑\\|少年负剑" C:/Code/JinShu-rust/src-tauri/target/debug/data/ 2>/dev/null`).toString().trim();
  console.log("   明文泄露检查:", leak ? "❌ 发现明文泄露 " + leak : "✅ 无明文泄露");
}

console.log("== 测试完成 ==");
ws.close();
process.exit(0);
