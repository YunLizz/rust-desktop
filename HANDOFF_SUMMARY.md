# 锦书 JinShu — 项目摘要 & 交接说明

> **换工具交接文件**。新 Agent 请从头读完再动手。
> 生成时间：2026-09-17（2026-10-18 完成 NSIS 安装包实测并更新）
> 详细上下文见同目录 `CONTEXT_HANDOFF.md`；项目介绍见 `README.md`。

---

## 一、一句话摘要

**锦书**是一款面向中文小说创作的本地桌面编辑器：Tauri v2（Rust 后端）+ Vue 3 + CodeMirror 6，全部稿件以 AES-256-GCM 加密存储在软件安装目录，支持 Windows 11 与 Arch Linux，Windows 便携包约 4.2MB。

---

## 二、Git 仓库信息（重点）

| 项 | 值 |
|---|---|
| **远程地址** | `https://github.com/YunLizz/rust-desktop.git` |
| **⚠️ 仓库名 ≠ 项目名** | 远程叫 `rust-desktop`，本地目录叫 `JinShu-rust`，同一个项目，别搞混 |
| 本地路径 | `C:\Code\JinShu-rust` |
| 分支 | `master`（本地与 `origin/master`）|
| 克隆命令 | `git clone https://github.com/YunLizz/rust-desktop.git JinShu-rust` |

### 认证方式
- **本机已配置 Git Credential Manager**（`credential.helper = manager`），`git fetch/push` **免密可用**，已实测通过。
- 换到新工具后，若工具自带终端在同一台机器上，直接 `git push` 即可，无需再输入密码。
- **凭据不落盘**：项目文件中不含任何账号/密码/token。若新工具环境没有凭据，请在系统凭据管理器或环境变量中配置，**不要写进代码或文档**。

### 当前 Git 状态（2026-10-18 更新）
```
本地 master : f60c302
远程 master : f60c302
状态        : 本地与远程完全一致，工作树干净（无未提交改动）
已推送      : f60c302 docs: 增加换工具交接摘要 + 定制 NSIS 安装器
              b84b0af docs: 补充仓库访问说明
              db61e18 docs: 增加跨对话上下文交接文档
```

> **NSIS 安装包已完成实测**（构建 / 安装 / 运行 / 加密落盘 / 卸载全部通过），详见第七节待办 1 与 `CONTEXT_HANDOFF.md` 第十一节。

---

## 三、技术栈与结构

```
JinShu-rust/
├── src-tauri/          Rust 后端（Tauri v2，~1900 行）
│   ├── src/main.rs         命令注册与入口（293 行）
│   ├── src/model.rs        数据模型（Novel/Chapter/Character/Task/TimelineEvent/OutlineNode）
│   ├── src/store.rs        加密存储、设置、数据目录解析
│   ├── src/crypto.rs       AES-256-GCM 加解密
│   ├── src/export.rs       导出 txt / md
│   ├── src/util.rs         字数统计等
│   ├── src/ai_client.rs    【已屏蔽】OpenAI/Anthropic 流式客户端（238 行）
│   └── src/ai_prompts.rs   【已屏蔽】14 个提示词模板（525 行）
├── frontend/           Vue 3 前端（~5100 行）
│   ├── src/store.js        全局状态（核心枢纽，含 window.__store 调试出口）
│   ├── src/api.js          Tauri invoke 封装
│   ├── src/names.js        本地起名算法（字库+规则）
│   ├── src/components/     Ribbon / TimelineTrack / LeftPane / RightPane /
│   │                       EditorView / DragHandle / Modal / Palette /
│   │                       ContextMenu / Icon / StatusBar
│   └── src/views/          Library / SettingsView / StatsView / RelationGraph
├── packaging/
│   ├── arch/PKGBUILD            Arch 打包
│   └── windows/installer.nsi    【未提交】定制 NSIS 安装器模板
├── tools/cdp_test.mjs      CDP 端到端测试脚本
├── dist/                   打包产物
├── CONTEXT_HANDOFF.md      详细上下文（10 章节）
├── CODE_WIKI.md / README.md
└── build_pkg_win.sh        Windows 打包脚本
```

### 常用命令
```bash
# 构建前端
cd frontend && npm install && npm run build

# 构建后端（自动嵌入前端产物）
cd src-tauri && cargo build --release
# 产物：src-tauri/target/release/jinshu.exe

# 测试（7 个，全部通过）
cd src-tauri && cargo test

# Windows 打包（前端构建 + 后端构建 + 压缩 zip）
bash build_pkg_win.sh

# 构建 NSIS 安装包（先构建前端，再打包；产物在 target/release/bundle/nsis/）
cd frontend && npm run build && cd ..
frontend/node_modules/.bin/tauri build --bundles nsis

# Arch 打包（需在 Linux 上）
cd packaging/arch && makepkg -si
```

---

## 四、⭐ 界面设计原则（用户核心主张，不要擅自改动）

用户按**人眼专注度**做空间分区（2K 27 寸屏）：

| 区域 | 内容 | 理由（用户原话） |
|---|---|---|
| **中央** | 正文 613px（屏宽 45% 且 ≤720px） | "双眼专注只能集中在屏幕中央 10 寸左右" |
| **左栏** | 5 棵树 Tab：章节/大纲/人物/世界/任务线 | "需要树形结构的显示" |
| **顶栏** | Ribbon 四选项卡 + 剪辑式横向时间轴 | "文件编辑设置 + 时间轴等横轴显示的功能" |
| **右栏** | 创意工具格：起名机/地图/关系网/时间轴/任务线 | "起名机地图等创意工具类" |

**硬约束**：
1. 正文**不能**铺满全屏（中文单行 30–40 字最佳）
2. 写作时**不翻页**（旧版 10 个互斥页面已被用户否决）
3. 三栏宽度可拖拽、可持久化

---

## 五、功能状态

### 已实现
书库 / 章节树（卷章层级）/ 大纲树（含"从章节生成骨架"）/ 人物树（按定位分组）+ 人物关系网 / 世界树 + 地图画布 / **任务线**（角色任务:归属角色+关联章节+任务链+公开暗线）/ **剪辑式横向时间轴**（分轨/拖动/缩放/类型配色）/ CodeMirror 编辑器（行号、4 字体、两端对齐、首行缩进、查找替换、宽度自适应）/ **本地起名机**（纯算法，6 风格组合）/ 导出 txt+md（md 含大纲）/ 深浅主题 + 8 强调色 / 命令面板 Ctrl+P / 自动保存 / 加密存储 / **Windows NSIS 安装包**（默认装到安装器所在目录，2026-10-18 实测通过）

### 已屏蔽（后端保留，前端移除）
**AI 功能** —— 用户明确说"**只是暂时屏蔽、后续还打算恢复**"：
- 后端 `ai_client.rs` / `ai_prompts.rs` / `ai_start|ai_cancel|ai_test` 命令 / `AiSettings` / `ureq` 依赖 **全部保留**
- 前端组件已删，恢复方式：
  ```bash
  git show b1873d4:frontend/src/prompts.js              # 提示词构建
  git show b1873d4:frontend/src/components/AIPanel.vue  # AI 面板
  ```
  再接回 `ai-chunk / ai-done / ai-error / ai-test-result` 事件监听。

### 已移除
`.jsb` 加密备份（`7d8e980`）、egui 原生版（`6586428`，因打包后中文字体加载失败而废弃）

---

## 六、安全模型（核心承诺）

- 所有作品/章节/设置（含 API Key）以 **AES-256-GCM** 加密文件存于**安装目录 `data/`**
- 磁盘上**不存在任何明文文件**（有测试 `store_encrypted_files` 保障）
- 密钥：`data/.jinshu_key`（随机 32 字节），指纹显示于状态栏
- **不写系统目录**（`%APPDATA%`/`~/.local`/`%TEMP%` 均不落盘）
- 数据目录优先级：`JINSHU_DATA_DIR` 环境变量 → exe 同级 `data/` → 用户目录兜底
- 定制 NSIS 模板正是为服务此承诺：默认安装目录改为 `$EXEDIR`（安装器所在目录），而非 `%LOCALAPPDATA%`

---

## 七、待办 / 已知问题

1. ~~提交并验证 NSIS 安装器~~ **已完成（2026-10-18 实测通过）**：`packaging/windows/installer.nsi` 已提交，构建、静默安装、运行、加密落盘、卸载全部验证；默认装到 `$EXEDIR\锦书`，不写 `%LOCALAPPDATA%`。详见 `CONTEXT_HANDOFF.md` 第十一节
2. **卸载向导「删除应用数据」勾选对锦书无效**：模板删的是 `%APPDATA%/%LOCALAPPDATA%\{bundleId}`，而真实数据在 `$INSTDIR\data`；可选修复（勾选时删除 `$INSTDIR\data`）
3. **Arch Linux 真机验证**（PKGBUILD 已写，未在 Linux 上跑过）
4. **AI 功能恢复**（用户已说明后续要做）
5. 时间轴"按章节分轨"时，无关联网章节的事件集中在"未关联章节"轨，可考虑支持手动指定轨道
6. 起名机字库可继续扩充（现约 300 字）

---

## 八、调试要点（踩过的坑）

| 场景 | 正确做法 |
|---|---|
| **GUI 自动化测试** | **WebView2 不支持 `mouse_event`/SendKeys 点击**，必须用 CDP：启动时设 `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS="--remote-debugging-port=9222"`，再用 WebSocket 连 `http://localhost:9222/json`。`window.__store` 已暴露可直接读写。 |
| **截图** | PowerShell 必须声明 `SetProcessDPIAware()`，否则 PrintWindow 得到缩放错误的图，会误判布局。 |
| **JS 静默错误** | 组件不渲染却无报错（如漏 import `ref`/`computed`）→ 用 CDP `Runtime.enable` 监听 `exceptionThrown`。 |
| **依赖下载失败** | 报 `CRYPT_E_REVOCATION_OFFLINE` 时，`src-tauri/.cargo/config.toml` 已设 `[http] check-revoke = false`。 |
| **保存失败** | 报 `missing field xxx` → 所有 model 已加 `#[serde(default)]`，新增字段务必跟随。 |
| **Vue 条件链** | 在 `v-if`/`v-else-if` 之间插组件会打断链导致后续区块不渲染，用 `<template>` 包裹。 |
| **NSIS 构建** | 本机 `%LOCALAPPDATA%\tauri\NSIS` 已缓存 makensis，直接 `tauri build --bundles nsis` 即可，无需联网；安装/卸载前需关闭正在运行的 JinShu（模板含 `CheckIfAppIsRunning`）。 |

---

## 九、给新 Agent 的工作建议

1. **先读** `CONTEXT_HANDOFF.md`（10 章节，含演化历史与设计决策全过程）
2. **不要擅自改 UI 布局** —— 用户对四区划分有强主张，改前用带预览图的选项确认
3. **报告先说结论，数据要实测**（本项目一贯用 CDP 实测 + 像素级验证 + 视觉子代理）
4. **凭据永不落盘**
5. **破坏性操作先确认**（删数据、改远程、强推）
6. 用户中文沟通，回中文
