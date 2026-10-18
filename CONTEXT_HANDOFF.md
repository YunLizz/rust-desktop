# 锦书 JinShu — 上下文交接文档

> **用途**：本文件用于跨对话继承项目上下文。新会话应先读本文，再动手。
> **最后更新**：2026-10-18
> **当前 HEAD**：`f60c302 docs: 增加换工具交接摘要 + 定制 NSIS 安装器`

---

## 一、项目是什么

**锦书** —— 面向中文小说创作的本地桌面编辑器。

| 项 | 值 |
|---|---|
| 路径 | `C:\Code\JinShu-rust` |
| 远程 | `https://github.com/YunLizz/rust-desktop.git`（**注意：仓库名是 rust-desktop，不是 JinShu-rust**）|
| 分支 | `master`（本地与 origin/master 同步）|
| 技术栈 | Tauri v2（Rust 后端）+ Vue 3 + CodeMirror 6 + Vite |
| 平台 | Windows 11 / Arch Linux 双平台 |
| 许可 | MIT |

### 仓库访问方式
- **本机已配置 git 凭据管理器**（`git config --get credential.helper` → `manager`），`git fetch` / `git push` **免密可用**，无需在项目文件里保存任何密码或 token。
- 新会话如需推送：直接 `git add` → `git commit` → `git push` 即可。
- ⚠️ **切勿把账号、密码、Personal Access Token 写入本项目任何文件**。凭据应只存在于系统凭据管理器（Windows Credential Manager）中。若确需在脚本里用 token，请用环境变量注入，不要落盘。

### 与用户的关系
用户是产品负责人，会亲自测试并给反馈。**用户对 UI/UX 有明确且强烈的设计主张**（见第四节），改动前应确认方向，不要自作主张改设计。

---

## 二、演化历史（为什么是现在这个样子）

| 阶段 | 提交 | 发生了什么 |
|---|---|---|
| 1 | `1ec1ad3` | 用 **Rust + egui** 从零实现（自绘 UI） |
| 2 | — | 发现问题：**egui 打包后中文字体加载失败，界面全崩**；且自绘 UI 做精细样式事倍功半 |
| 3 | — | 与用户商议后**重构为 Tauri**（Web 前端 + 复用 Rust 后端），包体从 25MB 降到 4.2MB |
| 4 | `b1873d4` | 融合原 Python 版（JInshu-py）特性（首行缩进/两端对齐/楷体仿宋/任务拖拽/章节骨架） |
| 5 | `7d8e980` | 用户决定：**移除 .jsb 加密备份**、**前端隐藏全部 AI 功能**（后端保留，仅屏蔽） |
| 6 | `ab73b5d` | **前端按「人眼专注区」重新设计**（当前架构的由来，详见第四节） |
| 7 | `37a403b` | 任务语义纠正为「**任务线**」+ 时间轴升级为「**剪辑式横向轨道**」 |
| 8 | `6586428` | 删除 egui 遗留代码，项目转为 Tauri 单引擎 |

> ⚠️ **egui 版已彻底删除**。若在旧对话/旧文档中看到 `src/*.rs`（根目录下的 Rust 源码）、`egui_code_editor` 依赖等，均为历史遗物，当前代码库不存在。

---

## 三、当前代码结构

```
JinShu-rust/
├── src-tauri/                    # Rust 后端（Tauri）
│   ├── src/
│   │   ├── main.rs       293行    # 命令注册 + 入口
│   │   ├── model.rs      294行    # 数据模型（Novel/Chapter/Character/Task/TimelineEvent…）
│   │   ├── store.rs      324行    # 加密存储 + 设置 + 数据目录解析
│   │   ├── crypto.rs      88行    # AES-256-GCM + Scrypt(遗)
│   │   ├── export.rs      92行    # 导出 txt/md
│   │   ├── util.rs        84行    # 字数统计、时间格式化
│   │   ├── ai_client.rs  238行    # 【已屏蔽】OpenAI/Anthropic 流式客户端
│   │   └── ai_prompts.rs 525行    # 【已屏蔽】14 个提示词模板
│   ├── capabilities/default.json  # Tauri 权限
│   └── tauri.conf.json            # 窗口/打包配置
├── frontend/                     # Vue 3 前端
│   ├── src/
│   │   ├── App.vue       188行    # 布局编排：Ribbon + 时间轴 + 三栏
│   │   ├── store.js               # 全局状态（核心枢纽）
│   │   ├── api.js                 # Tauri invoke 封装
│   │   ├── names.js               # 本地起名算法（字库+规则）
│   │   ├── components/
│   │   │   ├── Ribbon.vue         # 顶栏：分组选项卡 + 窗口控制
│   │   │   ├── TimelineTrack.vue  # 剪辑式横向时间轴
│   │   │   ├── LeftPane.vue       # 左栏 5 棵树
│   │   │   ├── RightPane.vue      # 右栏工具格
│   │   │   ├── EditorView.vue     # CodeMirror 编辑器
│   │   │   ├── DragHandle.vue     # 可拖拽分隔条
│   │   │   ├── Modal.vue          # 各类对话框
│   │   │   ├── Palette.vue        # Ctrl+P 命令面板
│   │   │   ├── ContextMenu.vue    # 全局右键菜单
│   │   │   ├── Icon.vue           # SVG 图标库
│   │   │   └── StatusBar.vue      # 底部状态栏
│   │   └── views/
│   │       ├── Library.vue        # 书库/欢迎页
│   │       ├── SettingsView.vue   # 设置（全屏）
│   │       ├── StatsView.vue      # 写作统计（全屏）
│   │       └── RelationGraph.vue  # 人物关系网（全屏）
│   └── dist/                      # 构建产物（已入库）
├── dist/JinShu-win64/             # Windows 便携包
├── packaging/
│   ├── windows/installer.nsi      # 定制 NSIS 安装器模板（默认装到 $EXEDIR）
│   └── arch/PKGBUILD              # Arch 打包
├── build_pkg_win.sh               # Windows 打包脚本
├── tools/cdp_test.mjs             # CDP 端到端测试脚本
├── README.md / CODE_WIKI.md       # 已同步
└── docs/screenshot.png
```

---

## 四、⭐ 界面设计原则（用户核心主张，务必遵守）

用户明确提出**按人眼专注度做空间分区**（2K 27 寸显示器，双眼专注区约屏幕中央 10 寸）：

| 区域 | 内容 | 设计理由（用户原话） |
|---|---|---|
| **中央** | 正文编辑区，**613px**（自适应：屏宽 45% 且 ≤720px） | "双眼专注只能集中在屏幕中央 10 寸左右，这个范围就是正文打字的区域" |
| **左栏** | **树形结构**：章节 / 大纲 / 人物 / 世界 / 任务线（5 棵树 Tab 切换） | "需要树形结构的显示" |
| **顶栏** | Ribbon 功能区 + **剪辑式横向时间轴** | "文件编辑设置等功能类 + 时间轴任务图谱等**横轴**显示的功能" |
| **右栏** | **创意工具格**：起名机 / 地图 / 关系网 / 时间轴 / 任务线 | "起名机地图等**创意工具类**的显示" |

### 关键约束
1. **正文绝不能铺满全屏** —— 中文单行 30-40 字最佳，过宽导致眼睛长距离横扫
2. **减少翻页** —— 旧版是 10 个互斥页面，写作时看人物要离开正文再切回，用户明确反对
3. **三栏宽度可拖拽**（拖分隔条 / 双击恢复默认 / 持久化到设置）
4. **窗口上 1/3 区域**用于展示参考类功能

---

## 五、功能清单与当前状态

### ✅ 已实现并验证

| 模块 | 说明 |
|---|---|
| 书库 | 欢迎页 + 作品卡片，双击打开，右键导出/删除 |
| 章节树 | 卷/章层级、筛选、字数统计、右键菜单（重命名/上移/下移/删除） |
| 大纲树 | 卷→章→节→要点；「从章节生成骨架」；AI 生成入口已移除 |
| 人物树 | 按角色定位分组、本章出现标记、头像色块 |
| 世界树 | 地点按类型分组（国家/城市/地区/建筑/异界） |
| **任务线** | **角色的任务**（非作者待办）：归属角色、关联章节、任务链、公开/暗线标记、按角色分组 |
| **剪辑式时间轴** | 按章节分轨/单轨平铺、事件块拖动排序、拖右边缘调长度、Ctrl+滚轮缩放、类型配色（主线/支线/暗线/回忆） |
| 编辑器 | CodeMirror 6、行号、当前行高亮、4 种字体（宋体/黑体/楷体/仿宋）、两端对齐、首行缩进、宽度自适应、查找替换、跨标签撤销 |
| **本地起名机** | 纯算法（`names.js`）：古风/仙侠/都市/西幻 4 风格 × 男/女/中性、人物名（含复姓）/书名/地名 |
| 工具格 | 地图画布（拖拽节点）、人物关系网（拖拽）、时间轴、任务线 |
| 导出 | txt / md（**md 含大纲**） |
| 主题 | 深色/浅色 + 8 种强调色预设 |
| 命令面板 | Ctrl+P，13 个命令 |

### 🚫 当前屏蔽 / 已移除

| 项 | 状态 | 恢复方式 |
|---|---|---|
| **AI 功能** | **前端已完全移除**（组件、事件、状态全删）；**后端完整保留**（`ai_client.rs`、`ai_prompts.rs`、3 个命令、`AiSettings`、`ureq` 依赖都在） | 用户明确说"**只是暂时屏蔽、后续还打算恢复**"。恢复时：1) 从 git 历史取回前端组件——`git show b1873d4:frontend/src/prompts.js`、`git show b1873d4:frontend/src/components/AIPanel.vue`；2) 接回 `ai-start/ai-chunk/ai-done/ai-error/ai-test-result` 事件监听 |
| .jsb 加密备份 | 已移除（后端 `export_jsb`/`import_jsb`、`scrypt`/`flate2` 依赖已删） | 如需恢复，参考 `7d8e980` 的父提交 |
| egui 原生版 | 已删除（`6586428`） | 不建议恢复（字体问题无解） |

---

## 六、安全模型（核心承诺，勿破坏）

- **加密缓存**：所有作品/章节/设置（含 API Key）以 **AES-256-GCM** 加密文件存于**安装目录 `data/`**，磁盘上**不存在任何明文文件**（有测试保障）
- **密钥**：首次运行生成 `data/.jinshu_key`（随机 32 字节），指纹显示在状态栏
- **不污染系统目录**：不写 `%APPDATA%`、`~/.local`、`%TEMP%`
- **数据目录解析优先级**：环境变量 `JINSHU_DATA_DIR` → exe 同级 `data/` → 用户目录兜底
- **安全边界**：防"随手读盘"，不防同权限定向攻击（已在 README 说明）

---

## 七、开发与调试

### 构建
```bash
# 前端
cd frontend && npm install && npm run build
# 后端（自动嵌入前端产物）
cd src-tauri && cargo build --release
# 产物：src-tauri/target/release/jinshu.exe
```

### 打包
```bash
bash build_pkg_win.sh          # Windows 便携 zip → dist/JinShu-rust-win64-tauri.zip（~4.1MB）

# Windows NSIS 安装包（先构建前端；产物在 src-tauri/target/release/bundle/nsis/）
cd frontend && npm run build && cd ..
frontend/node_modules/.bin/tauri build --bundles nsis

cd packaging/arch && makepkg -si   # Arch（需在 Linux 上执行）
```

### 测试
```bash
cd src-tauri && cargo test     # 7 个测试，全部通过
```
| 测试 | 验证内容 |
|---|---|
| `crypto::crypto_roundtrip` | 加密往返、错误密钥拒绝、错误 AAD 拒绝、密文不含明文 |
| `store::store_encrypted_files` | **磁盘零明文**（含标题与正文）、密钥文件生成、读回一致性 |
| `export::export_formats` | txt/md 导出内容与结构 |
| `model::word_count_cjk` | 中英混排字数统计规则 |
| `model::novel_chapter_flow` | 卷章增删流程 |
| `ai_prompts::outline_parse` | AI 大纲文本解析（该函数被导出复用，故测试保留） |

### ⭐ CDP 调试（本次会话的核心工具）
**WebView2 不支持 `mouse_event` 注入**，GUI 自动化必须走 CDP：

```bash
# 启动时开调试端口
WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS="--remote-debugging-port=9222" ./jinshu.exe
```
然后可用 `tools/cdp_test.mjs`（或临时 node 脚本）通过 WebSocket 驱动：
```js
const list = await fetch("http://localhost:9222/json").then(r=>r.json());
const ws = new WebSocket(list.find(p=>p.type==="page").webSocketDebuggerUrl);
// Runtime.evaluate 执行 JS、Page.captureScreenshot 截图、Input.insertText 输入
// Runtime.enable 后监听 exceptionThrown —— 定位静默 JS 错误的关键手段
```
`window.__store` 已暴露全局状态，可直接读写调试。

### 截图
PowerShell 截图**必须声明 DPI 感知**（否则拿到缩放错误的图）：
```powershell
[DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
# 再用 PrintWindow(hWnd, hdc, 2) 抓窗口
```

---

## 八、坑与教训（避免重蹈覆辙）

| 坑 | 现象 | 解决 |
|---|---|---|
| **egui 打包字体丢失** | 打包后中文全部消失（只剩微点） | 弃用 egui，转 Tauri |
| **WebView2 拒绝鼠标注入** | `mouse_event`/`SendKeys` 点击无效，界面无反应 | 用 CDP 驱动 |
| **PowerShell 截图 DPI 错误** | 截图尺寸/内容与实际不符，误判"布局错位" | 声明 `SetProcessDPIAware()` |
| **JS 导入遗漏静默失败** | 组件不渲染但无报错（如 `ref`/`computed` 未导入） | 用 CDP 监听 `exceptionThrown` |
| **Vue 条件链被打断** | 插入 `v-if` 组件后，后面的 `v-else-if` 区块永不渲染（正文区消失） | 用 `<template>` 包裹 |
| **serde 过于严格** | 前端字段缺失 → 整个 `save_novel` 失败 `missing field xxx` | 所有 model 加 `#[serde(default)]`；`Option<T>` 加默认值函数 |
| **cargo 网络证书吊销检查** | `CRYPT_E_REVOCATION_OFFLINE` 导致依赖下载失败 | `src-tauri/.cargo/config.toml` 设 `[http] check-revoke = false` |
| **Python 脚本转义层级** | `\n` 在模板字符串里被解释成真实换行，破坏 Vue 模板 | 用计算属性替代模板内联表达式 |
| **中文字符串切片** | `body[idx+1..]` 切在多字节字符中间 panic | 用 `char_indices()` + `len_utf8()` |

---

## 九、当前状态与待办

### 运行状态
- 应用未运行（安装包验证结束后已关闭）
- 2026-10-18 验证：NSIS 安装包构建 / 安装 / 运行 / 加密落盘 / 卸载全部通过（详见第十一节）
- 此前验证：编译通过、启动零异常、全部界面元素正常

### 已知待办 / 可改进
1. **AI 功能恢复**（用户已说明后续要做，后端已就绪）
2. **Arch Linux 实际构建验证** —— PKGBUILD 已写但**未在真机验证过**
3. ~~NSIS 安装包~~ **已完成（2026-10-18 实测，见第十一节）**
4. **卸载向导「删除应用数据」勾选对锦书无效** —— 模板删的是 `%APPDATA%/%LOCALAPPDATA%\{bundleId}`，真实数据在 `$INSTDIR\data`；可选修复（勾选时删除 `$INSTDIR\data`）
5. 时间轴「按章节分轨」模式下，未关联网章节的事件集中在"未关联章节"轨，可考虑支持手动指定轨道
6. 起名机字库可继续扩充（现约 300 字）

### 工作区状态
`git status` **干净**，本地与 `origin/master` 同步（`f60c302`）。

---

## 十、沟通偏好（重要）

- 用户是中文母语，**所有回复用中文**
- 用户会亲自测试并给具体反馈，**不要替他做设计决策**，涉及 UI/UX 时先确认
- 用户重视「**减少翻页、提升写作专注度**」，任何改动都应服务于此
- 遇到不确定的设计问题，**用 AskUserQuestion 给带预览图的选项**（用户接受度良好）
- 报告要**先说结论**，数据要**实测**（本项目大量使用 CDP 实测 + 像素级验证）
- 破坏性操作（删文件、改远程）必须先确认

---

## 十一、NSIS 安装包实测记录（2026-10-18）

### 环境与构建
- 会话以完全权限运行（此前 workspace-write 沙箱因 `C:\Code` 缺少修改权限位而无法启动命令行）
- NSIS 工具链已缓存于 `%LOCALAPPDATA%\tauri\NSIS`，makensis 可用，无需联网下载
- 命令：`cd frontend && npm run build` → `frontend/node_modules/.bin/tauri build --bundles nsis`
- 产物：`src-tauri/target/release/bundle/nsis/锦书_0.1.0_x64-setup.exe`（3,098 KB，`target/` 已 gitignore，未入库）

### 实测结果
| 项 | 结果 |
|---|---|
| 默认安装目录 | `$EXEDIR\锦书`（安装器放 `C:\Code\_installer_verify\` 时装到 `C:\Code\_installer_verify\锦书`）；**未创建** `%LOCALAPPDATA%\锦书` |
| `%APPDATA%` / `%TEMP%` | 均未创建锦书目录 |
| 自定义目录 | `setup.exe /S /D=<dir>` 正常，不会误建默认目录 |
| 快捷方式 / 注册表 | 开始菜单 + 桌面快捷方式、`HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\锦书` 均正常 |
| 启动 | 标题 `锦书 · 小说编辑器`，1380×880，CDP `Runtime.enable` 全程零 `exceptionThrown` |
| 数据落盘 | 安装目录 `data/`：`.jinshu_key`(32B)、`settings.jsr`、`novels/<id>/novel.jsr`、`chapters/<id>.jsr`（JSR1 格式） |
| 加密承诺 | 新建作品并输入正文 → 章节文件 137B→215B；对标题/正文/唯一标记做 UTF-8 字节级扫描，**磁盘零明文** |
| 卸载 | `/S` 卸载后 exe、桌面/开始菜单快捷方式、卸载注册表项全部删除；`data/` 保留（作品不丢） |
| 已知问题 | 卸载向导「删除应用数据」勾选删除 `%APPDATA%/%LOCALAPPDATA%\{bundleId}`，对 `$INSTDIR\data` 无效（见待办 4） |

### 复现要点
- 启动程序时设 `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222`，再用 node 脚本连 `http://127.0.0.1:9222/json` 驱动（参考 `tools/cdp_test.mjs`）
- 静默安装 `setup.exe /S`、静默卸载 `uninstall.exe /S`；`$EXEDIR` 即安装器所在目录
- 截图与临时脚本在会话工作区 `C:\Code\_installer_verify\`（非项目文件，可删）
