# 锦书 JinShu-rust · Code Wiki

> 为中文小说创作而生的本地编辑器，Rust + Tauri 2（Vue 3 前端），全部数据以 AES-256-GCM 加密缓存于安装目录。

> **文档对应代码版本**：仓库 HEAD（Tauri 单引擎，2026-09-16）。
>
> **最后同步：2026-09-16（前端重构后同步：Ribbon + 左栏 5 树 + 右栏创意工具格）**

> **单引擎说明**：本项目现仅有 **Tauri 版**一套实现（`src-tauri/` 后端 + `frontend/` 前端，官方推荐发布版）。原「egui 原生版」（根 `src/` 目录、根 `Cargo.toml`、`assets/` 资源、`build_pkg_arch.sh`）已于 2026-09 移除，相关代码与资源均已不存在。
>
> 当前生效的能力边界（均为 Tauri 版）：
>
> - **无 AI 前端**：AI 面板、命令面板 AI 命令、编辑区「续写」、人物卡「AI 完善人设」、状态栏 AI 状态、设置页「AI 服务」段、`Ctrl+J` 与 `ai-*` 事件监听全部下线；后端 `ai_client.rs` / `ai_prompts.rs` 与 `ai_start` / `ai_cancel` / `ai_test` 命令保留但**无调用方**（`dead_code` 警告属预期）。
> - **无 `.jsb` 备份**：`.jsb` 加密备份的导出与导入均已移除，导出仅保留 txt / md。
> - **唯一落盘格式**：JSR1（AES-256-GCM），密钥为 `data/.jinshu_key`。

---

## 目录

1. [项目总览与技术栈](#1-项目总览与技术栈)
2. [整体架构设计](#2-整体架构设计)
3. [目录结构说明](#3-目录结构说明)
4. [数据模型层 (Model)](#4-数据模型层-model)
5. [加密存储层 (Store / Crypto)](#5-加密存储层-store--crypto)
6. [AI 服务层 (AI)（后端保留，前端未接入）](#6-ai-服务层-ai)
7. [Tauri 后端 (src-tauri/)](#7-tauri-后端-src-tauri)
8. [Vue 3 Web 前端 (frontend/)](#8-vue-3-web-前端-frontend)
9. [导出模块](#9-导出模块)
10. [工具函数 (util.rs)](#10-工具函数-utilrs)
11. [模块依赖关系图](#11-模块依赖关系图)
12. [核心数据流](#12-核心数据流)
13. [构建与运行方式](#13-构建与运行方式)
14. [配置与环境变量](#14-配置与环境变量)
15. [测试覆盖](#15-测试覆盖)
16. [快捷键与命令系统](#16-快捷键与命令系统)

---

## 1. 项目总览与技术栈

### 1.1 项目定位

**锦书 (JinShu-rust)** 是一款面向中文小说创作者的本地 IDE 风格编辑器，核心特性：

| 能力 | 实现方式 |
|------|----------|
| 界面布局 | Ribbon（写作/设定/工具/视图 4 选项卡）+ 左栏 5 树 + 中央正文 + 右栏创意工具格 |
| 章节 / 分卷管理 | 卷→章 树形结构，多标签页，拖拽排序 |
| 写作编辑器 | CodeMirror 6 |
| 大纲系统 | 卷→章→节→要点 四级树形大纲 |
| 人物设定 | 人物卡 + 关系网画布（可拖拽） |
| 世界观 | 地点层级树（国家→城市→建筑） |
| 时间线 | 事件排序 + 关联章节/人物/地点 |
| 任务线 | 任务链 + 三列看板（未开始/进行中/已完成），归属角色、关联章节、明暗线 |
| 写作统计 | 总字数 / 今日 / 连续天数 / 30天趋势 |
| AI 创作助手 | 前端已全部下线（面板/入口/设置项均已移除）；后端代码保留待重新设计 |
| Lorebook 注入 | 随 AI 前端一并下线，无前端入口（后端提示词模板中仍保留匹配逻辑） |
| 数据安全 | AES-256-GCM 加密落盘，不写系统目录 |
| 导出格式 | txt / md；`.jsb` 加密备份已移除 |

### 1.2 技术栈全景

```
┌─────────────────────────────────────────────────────────────────┐
│                        用户界面层                                │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  Tauri 2 + Vue 3 + Vite 6  (src-tauri/ + frontend/)        │  │
│  │  CodeMirror 6 编辑器                                       │  │
│  └───────────────────────────────────────────────────────────┘  │
└───────────────────────────────┬─────────────────────────────────┘
                                │
┌───────────────────────────────▼─────────────────────────────────┐
│                      业务逻辑层（Rust）                          │
│  ┌───────────┐  ┌───────────┐  ┌───────────┐  ┌───────────┐     │
│  │   Model   │  │   Store   │  │    AI     │  │  Export   │     │
│  │  数据模型  │  │  加密存储  │  │ 流式客户端 │  │   导出     │     │
│  │           │  │           │  │(前端无入口)│  │           │     │
│  └───────────┘  └───────────┘  └───────────┘  └───────────┘     │
└─────────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────────┐
│                       基础设施层                                 │
│  AES-256-GCM (aes-gcm)  |  ureq (HTTP，仅 AI)                    │
│  UUID v4  |  Chrono  |  Serde JSON  |  rand (CSPRNG)             │
│  SHA-256 (密钥指纹)                                              │
└─────────────────────────────────────────────────────────────────┘
```

### 1.3 引擎说明

本项目为 **Tauri 单引擎**实现，仅维护一套前端：

| 版本 | 路径 | UI 框架 | 说明 |
|------|------|---------|------|
| Tauri 版 | `src-tauri/` + `frontend/` | Tauri 2 + Vue 3 | 唯一实现，官方推荐发布版本 |

> 原 egui 原生版（根 `src/`、根 `Cargo.toml`、`build.rs`、`build_pkg_arch.sh`、`assets/`）已于 2026-09-16 随代码一并删除，不再维护。

---

## 2. 整体架构设计

### 2.1 分层架构

```
┌───────────────────────────────────────────────────────┐
│                    Presentation (UI)                   │
│  Vue Components ─ store.js ── Tauri invoke            │  ← frontend/
├───────────────────────────────────────────────────────┤
│                   Application Layer                    │
│  Tauri Commands (命令接口层，前端 ↔ 后端桥接)          │
├───────────────────────────────────────────────────────┤
│                     Domain Layer                       │
│  Novel · Volume · ChapterMeta · OutlineNode           │
│  Character · Location · TimelineEvent · Task          │
│  业务方法: add_chapter / delete_chapter / total_words  │
├───────────────────────────────────────────────────────┤
│                  Infrastructure Layer                  │
│  Store (加密IO) · Crypto (AES-256-GCM / JSR1)         │
│  AI Client (流式HTTP，前端无调用入口)                   │
│  Export (仅 txt/md) · Util (ID/时间/字数)              │
└───────────────────────────────────────────────────────┘
```

### 2.2 设计原则

1. **数据本地优先**：所有数据加密切块存在安装目录，不写 `%APPDATA%`/`~/.local`/临时目录
2. **密钥与数据解耦**：本地密钥 `.jinshu_key`（32 字节随机，仅本机可解密）
3. **无状态AI客户端**：流式 SSE 输出，经 Tauri events 推送增量（**当前前端已无调用入口**）
4. **内存态 + 定期刷盘**：正文在前端 `store.chapters`，脏位标记 + 自动保存（后端默认 5 秒 / 前端 2 秒 debounce）
5. **前后端解耦**：前端只经 IPC 调用命令层，不直接触碰文件系统与密钥

---

## 3. 目录结构说明

```
JinShu-rust/
├── frontend/                   # Vue 3 前端（Vite 构建，src/ 共 21 个源文件）
│   ├── src/
│   │   ├── components/         #   UI 组件（11 个）
│   │   │   ├── Ribbon.vue          #   顶部两行：选项卡 + 窗口控制 + 当前组按钮 + 时间轴折叠钮
│   │   │   ├── LeftPane.vue        #   左栏：5 个 Tab 树 + 过滤 + 右键菜单
│   │   │   ├── RightPane.vue       #   右栏：创意工具格 + 5 个内嵌工具面板
│   │   │   ├── TimelineTrack.vue   #   顶栏剪辑式横向时间轴（可折叠、分轨、拖拽、Ctrl+滚轮缩放）
│   │   │   ├── DragHandle.vue      #   通用可拖拽分隔条（双击复位、宽度落盘）
│   │   │   ├── EditorView.vue      #   中央编辑器：标签页 + 查找替换条 + CodeMirror 正文
│   │   │   ├── StatusBar.vue       #   底部状态栏
│   │   │   ├── Palette.vue         #   命令面板（13 条命令）
│   │   │   ├── Modal.vue           #   通用弹窗（8 种 kind）
│   │   │   ├── ContextMenu.vue     #   自定义右键菜单（jinshu:contextmenu 事件）
│   │   │   └── Icon.vue            #   SVG 图标组件
│   │   ├── views/              #   全屏视图（store.fullView）
│   │   │   ├── Library.vue         #   书库列表 + 空书库欢迎页
│   │   │   ├── StatsView.vue       #   写作统计
│   │   │   ├── SettingsView.vue    #   设置页（外观 / 编辑 / 存储与安全）
│   │   │   └── RelationGraph.vue   #   全屏人物关系网画布
│   │   ├── store.js            #   全局响应式状态 + 业务动作
│   │   ├── api.js              #   Tauri invoke 命令封装（12 个）
│   │   ├── names.js            #   本地起名机（纯算法字库，不依赖网络）
│   │   ├── App.vue             #   根组件：Ribbon + 全屏视图 + 写作台三栏 + StatusBar + 浮层
│   │   ├── main.js             #   入口：initInfo/恢复上次作品、全局 keydown 快捷键、挂载
│   │   └── styles/theme.css    #   CSS 变量主题系统（深浅色 + 8 种强调色）
│   ├── index.html
│   ├── package.json            #   Vue 3.5 + Vite 6 + CodeMirror 6
│   └── vite.config.js
│
├── src-tauri/                  # Rust 后端
│   ├── src/
│   │   ├── main.rs             #   Tauri Builder + 所有 #[tauri::command]
│   │   ├── model.rs            #   数据模型 + 业务方法
│   │   ├── store.rs            #   加密存储 IO
│   │   ├── crypto.rs           #   AES-256-GCM（JSR1）
│   │   ├── ai_client.rs        #   流式 AI 客户端（前端无调用方）
│   │   ├── ai_prompts.rs       #   提示词模板（前端无调用方）
│   │   ├── export.rs           #   导出（仅 txt / md）
│   │   └── util.rs             #   工具函数
│   ├── Cargo.toml              #   tauri 2 + tauri-plugin-dialog/opener
│   ├── build.rs                #   tauri-build 嵌入前端 dist
│   ├── tauri.conf.json         #   Tauri 2 配置（窗口/权限/标识符）
│   ├── capabilities/           #   权限清单（default.json）
│   ├── icons/                  #   多尺寸图标
│   └── gen/                    #   Tauri 生成的 schema
│
├── packaging/                  # 系统打包
│   └── arch/PKGBUILD           #   Arch Linux AUR 风格包
├── dist/                       # 发布产物（已打包的便携版）
├── tools/
│   └── cdp_test.mjs            #   CDP 浏览器自动化测试脚本
├── docs/screenshot.png         #   README 截图
├── build_pkg_win.sh            #   Windows 便携版打包脚本
├── LICENSE (MIT)
└── README.md
```

> **已删除的旧前端组件**（前端重构 `ab73b5d` + `37a403b`「按人眼专注区重新设计」后已不存在）：`components/ActivityBar.vue`、`components/SidePanel.vue`、`components/TitleBar.vue`、`views/DetailViews.vue`。旧导航字段 `store.activity` 同步废弃（其职责由 `ribbonTab` / `leftTree` / `rightTool` / `fullView` 分担）。

---

## 4. 数据模型层 (Model)

> 路径：[src-tauri/src/model.rs](file:///c:/Code/JinShu-rust/src-tauri/src/model.rs)

### 4.1 核心结构体关系

```
Novel (小说根对象)
├── meta: NovelMeta            # 元信息（id/标题/作者/简介/字数…）
├── volumes: Vec<Volume>       # 分卷列表
│   └── Volume
│       ├── id, title
│       └── chapters: Vec<ChapterMeta>  # 章节点（不含正文）
├── outline: Vec<OutlineNode>  # 树形大纲（卷→章→节→要点）
├── characters: Vec<Character> # 人物卡片
├── locations: Vec<Location>   # 世界观地点
├── timeline: Vec<TimelineEvent> # 时间线事件
├── tasks: Vec<Task>           # 任务
├── chains: Vec<TaskChain>     # 任务链分组
└── stats: BTreeMap<String, u64>  # 日期→当日新增字数
```

### 4.2 结构体详解

> 全部 **11 个结构体**均标注了 `#[serde(default)]`（`NovelMeta` / `ChapterMeta` / `Volume` / `OutlineNode` / `Relationship` / `Character` / `Location` / `TimelineEvent` / `Task` / `TaskChain` / `Novel`），字段缺失时回退默认值，避免旧数据因缺字段而保存失败。

#### NovelMeta — 小说元信息

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | String | UUID v4（simple格式，32位无连字符） |
| `title` | String | 书名 |
| `author` | String | 作者 |
| `description` | String | 简介 |
| `genre` | String | 题材（玄幻/都市/…） |
| `created_at` | i64 | 创建时间戳（秒） |
| `updated_at` | i64 | 最近修改时间戳 |
| `total_words` | u64 | 总字数（各 ChapterMeta.words 之和） |
| `chapter_count` | u32 | 章节总数 |

#### Volume / ChapterMeta — 卷章树

```rust
pub struct Volume {
    pub id: String,
    pub title: String,                     // 如 "第一卷 · 出山"
    pub chapters: Vec<ChapterMeta>,
}
pub struct ChapterMeta {
    pub id: String,
    pub title: String,
    pub words: u64,                        // 保存时重算
    pub updated_at: i64,
}
```

#### Character — 人物卡

| 字段 | 类型 | 说明 |
|------|------|------|
| `role` | String | 主角 / 重要配角 / 配角 / 反派 / 其他 |
| `appearance` | String | 外貌描述 |
| `personality` | String | 性格 |
| `background` | String | 背景故事 |
| `goals` | String | 动机与目标 |
| `notes` | String | 备注 |
| `relationships` | Vec<Relationship> | 关系列表 → (target_id, target_name, relation, note) |

#### Location — 世界观地点

- `parent_id: Option<String>` 支持嵌套层级（国家→省→城市→建筑）
- `kind` 枚举：国家 / 城市 / 地区 / 建筑 / 异界 / 其他

#### TimelineEvent — 时间线事件

```rust
pub struct TimelineEvent {
    pub id: String,
    pub title: String,
    pub time: String,                     // 自由描述（如 "第三卷 第12章 前夜"）
    pub description: String,
    pub character_ids: Vec<String>,       // 关联人物
    pub location_id: Option<String>,      // 关联地点
    pub chapter_id: Option<String>,       // 关联章节
    pub order: i32,                       // 剪辑式轨道的横向排序位置（拖动调整）
    pub kind: String,                     // 事件类型：主线 / 支线 / 暗线 / 回忆（决定轨道颜色，默认「主线」）
}
```

#### Task / TaskChain — 任务线

Task 字段（`src-tauri/src/model.rs`）：

| 字段 | 类型 | 说明 |
|------|------|------|
| `status` | u8 | **0 未开始 / 1 进行中 / 2 已完成** |
| `chain_id` | Option<String> | 关联 TaskChain；未关联的归入 "无分组" |
| `character_id` | Option<String> | 归属角色（「角色的任务」，不是作者的待办） |
| `chapter_id` | Option<String> | 关联章节（任务在哪一章推进/完成） |
| `is_public` | bool | 默认 true；**false = 暗线/伏笔**（读者尚未知晓） |
| `order` | i32 | 在任务线列表中的排序 |

### 4.3 Novel 关键业务方法

| 方法 | 签名 | 说明 |
|------|------|------|
| `new()` | `(title, author, desc, genre) -> Novel` | 创建新小说（分配 id + ts） |
| `chapters_all()` | `() -> Vec<&ChapterMeta>` | 扁平化遍历所有卷的所有章 |
| `find_chapter[_mut]()` | `cid -> Option<&ChapterMeta>` | O(N) 按 id 查找章 |
| `volume_of_chapter()` | `cid -> Option<&Volume>` | 反查所属卷 |
| `add_volume()` | `title -> ()` | 新建卷 |
| `add_chapter()` | `Option<vid>, title -> cid` | 新建章；无卷时自动创建"正文"卷 |
| `delete_chapter()` | `cid -> ()` | 删除章 + 清理关联时间线事件 |
| `total_words()` | `() -> u64` | 所有章 words 求和 |
| `sync_from_chapters()` | `() -> ()` | 旧数据兼容：空卷时按章列表重建默认卷 |

---

## 5. 加密存储层 (Store / Crypto)

> 路径：[src-tauri/src/store.rs](file:///c:/Code/JinShu-rust/src-tauri/src/store.rs) + [src-tauri/src/crypto.rs](file:///c:/Code/JinShu-rust/src-tauri/src/crypto.rs)

### 5.1 数据目录解析

优先级从高到低：
1. 环境变量 `JINSHU_DATA_DIR`
2. 可执行文件同目录 `data/`（便携模式）
3. 兜底：`$HOME/.jinshu/data`（Unix）或 `%USERPROFILE%/.jinshu/data`（Win）

### 5.2 目录结构

```
data/
├── .jinshu_key              # 32字节随机密钥（首次生成）
├── settings.jsr             # AppSettings（加密）
└── novels/
    └── {novel_id}/
        ├── novel.jsr        # Novel 序列化（加密，含元信息+大纲+人物+…）
        └── chapters/
            ├── {cid_1}.jsr  # 章节正文（加密，纯UTF-8字节）
            └── {cid_2}.jsr
```

- `.jsr` = JinShu Rust 本地加密格式（**当前唯一落盘格式**）
- `.jsb` = JinShu Backup 跨设备密码备份格式（**已移除**，导出/导入能力均不存在）

### 5.3 加密方案

#### JSR 本地格式（AES-256-GCM）

```
文件布局: [MAGIC 4B][VER 1B][NONCE 12B][CIPHERTEXT...]
          J S R 1    0x01    随机数      AES-256-GCM(plain, AAD=filename)
```

- **密钥来源**：`data/.jinshu_key`（32B，首次启动 `rand::thread_rng()` 生成）
- **AAD**：文件名字节（绑定文件名，防止密文被改名替换）
- **原子写入**：先写 `{path}.tmp`，成功后 `rename` 替换，避免半写损坏
- **Unix**：密钥目录与密钥文件权限收紧为 `0o700` / `0o600`

核心函数：
```rust
pub fn encrypt_bytes(key: &[u8;32], aad: &[u8], plain: &[u8]) -> Result<Vec<u8>, String>
pub fn decrypt_bytes(key: &[u8;32], aad: &[u8], data: &[u8]) -> Result<Vec<u8>, String>
pub fn encrypt_file(path: &Path, key: &[u8;32], plain: &[u8]) -> Result<(), String>
pub fn decrypt_file(path: &Path, key: &[u8;32]) -> Result<Vec<u8>, String>
pub fn fingerprint(key: &[u8;32]) -> String  // SHA256前12位十六进制（状态栏显示）
```

### 5.4 Store 结构体 API

```rust
pub struct Store {
    pub data_dir: PathBuf,
    pub key: [u8; 32],
}

// 生命周期
pub fn Store::init() -> Result<Store, String>   // 解析目录 + 加载/生成密钥

// 设置 IO
pub fn load_settings(&self) -> AppSettings      // 解密 settings.jsr，失败回退 Default
pub fn save_settings(&self, &AppSettings) -> Result<(), String>

// 小说 IO
pub fn list_novels(&self) -> Vec<NovelMeta>     // 扫描 novels/ 目录，解密 novel.jsr 取 meta
pub fn load_novel(&self, id) -> Result<Novel, String>
pub fn save_novel(&self, &Novel) -> Result<(), String>

// 章节 IO（单文件加密）
pub fn load_chapter(&self, novel_id, cid) -> Result<String, String>
pub fn save_chapter(&self, novel_id, cid, text) -> Result<(), String>
pub fn delete_chapter_file(&self, novel_id, cid)
pub fn delete_novel(&self, id) -> Result<(), String>  // 递归 rm -rf 小说目录
```

### 5.5 AppSettings 结构

```
AppSettings
├── theme: "dark" | "light"
├── accent: [u8;3]               # RGB 强调色（8种预设）
├── ui_scale: f32                # 界面缩放
├── autosave_secs: u64           # 自动保存间隔（默认5秒）
├── last_novel_id: Option<String># 下次启动自动打开
├── sidebar_width, ai_panel_width: f32
├── nav_expanded: Option<bool>   # 保留字段（默认 Some(true)），前端已无入口
├── recent: Vec<RecentNovel>     # 最近打开（最多8项）
├── editor: EditorSettings
│   ├── font: "serif" | "sans"
│   ├── font_size: f32           # 默认 17
│   ├── line_spacing: f32        # 默认 1.9
│   ├── wrap: bool               # 默认 true
│   ├── markdown_highlight: bool # 中文写作默认关闭
│   ├── auto_indent: bool        # 首段自动两字缩进
│   ├── show_line_numbers: bool  # 默认 true
│   ├── justify: bool            # 两端对齐（中文排版，默认 true）
│   └── line_width_chars: u32    # 正文每行目标字数（控制列宽，默认 34）
└── ai: AiSettings              # 字段保留用于兼容既有 settings.jsr；前端已无任何入口
    ├── protocol: "openai" | "anthropic"
    ├── base_url
    ├── api_key                 # 加密存储
    ├── model                   # 默认 gpt-4o-mini
    ├── temperature             # 默认 0.8
    ├── max_tokens              # 默认 4096
    ├── system_prompt           # 默认：资深中文小说创作助手…
    ├── timeout_secs            # 默认 180
    └── inject_lore: bool       # 自动 Lorebook 注入开关
```

> `AppSettings.ai`（`AiSettings`）与 `AppSettings.nav_expanded` 均**保留未删**，用于兼容既有 `settings.jsr`，但前端已无任何入口/消费方。

---

## 6. AI 服务层 (AI)

> 路径：[src-tauri/src/ai_client.rs](file:///c:/Code/JinShu-rust/src-tauri/src/ai_client.rs) + [src-tauri/src/ai_prompts.rs](file:///c:/Code/JinShu-rust/src-tauri/src/ai_prompts.rs)

> ⚠️ **接入状态（2026-09-09 起，提交 `7d8e980`）**
>
> - 前端 AI 入口已**全部下线**（`AIPanel.vue`、`prompts.js` 整文件删除；命令面板 AI 命令、编辑区「续写」、侧栏「AI 生成」、人物卡「AI 完善人设」、状态栏 AI 状态、设置页「AI 服务」段、`Ctrl+J`、`ai-*` 事件监听全部移除）。
> - `ai_start` / `ai_cancel` / `ai_test` 三条命令**仍在 `invoke_handler` 中注册**，但已无任何调用方；`ai_client.rs` / `ai_prompts.rs` 随之产生 `dead_code` 警告，属**预期现象**，保留作为后续 AI 大改的起点。
> - `store::AiSettings`、`AppSettings.ai`、`Model.ai_summary` 字段同样保留，用于兼容既有 `settings.jsr` 数据。

### 6.1 架构：独立线程 + SSE 流式 + 事件推送

```
Worker Thread (ureq Agent)
  ├── stream_openai()    ──► POST /chat/completions ?stream=true
  └── stream_anthropic() ──► POST /v1/messages ?stream=true
      │
      └── 按行解析 SSE → AiEvent::Chunk(text) 回传
          完成 → AiEvent::Done
          错误 → AiEvent::Error(msg)
```

Rust 侧通过 `app.emit()` 推送前端事件：
- `ai-chunk` / `ai-done` / `ai-error` / `ai-test-result`
- ⚠️ 前端**已不再注册这些监听器**（原注册代码位于 `frontend/src/main.js`，已移除）

### 6.2 AiEvent 枚举

```rust
pub enum AiEvent {
    Chunk(String),     // 增量文本片段
    Done,              // 正常结束
    Error(String),     // 网络/协议/取消
}
```

### 6.3 公共入口

```rust
pub fn stream_chat(
    cfg: &AiSettings,
    messages: &[(String, String)],   // [(role, content)] 不含 system
    tx: Sender<AiEvent>,
    cancel: Arc<AtomicBool>,         // 外部可随时取消
)
```

Tauri 命令（**已注册但当前无前端调用方**）：
```rust
#[tauri::command] ai_start(cfg, messages)  // 启动流式请求，结果走事件
#[tauri::command] ai_cancel()              // 置位 cancel flag
#[tauri::command] ai_test(cfg)             // 非流式短连接，检查 API Key 配置
```

### 6.4 协议差异

| 项目 | OpenAI 兼容 | Anthropic |
|------|-------------|-----------|
| URL | `{base}/chat/completions` | `{base}/v1/messages` |
| 认证 Header | `Authorization: Bearer {key}` | `x-api-key: {key}` + `anthropic-version` |
| System Prompt | 首条 `role=system` | 顶层 `system` 字段 |
| SSE delta 路径 | `choices[0].delta.content` | `content_block_delta.delta.text` |
| 完成标记 | `data: [DONE]` | 流关闭 |

### 6.5 提示词系统 (prompts.rs)

**14 种 AI 动作**：续写、全本大纲、章节细纲、润色、扩写、章节摘要、剧情提示、逻辑检查、一致性检查、整稿评审、人物卡、世界观、起名、生成简介。

关键辅助函数：
```rust
pub fn outline_to_text(novel: &Novel) -> String        // 大纲树转缩进文本
pub fn characters_to_text(novel: &Novel, names: &[String]) -> String  // 指定人物卡文本
pub fn world_to_text(novel: &Novel) -> String          // 世界观设定文本
pub fn lore_hits(novel: &Novel, text: &str) -> (Vec<String>, Vec<String>)
  // 从正文末尾 N 字符中扫描，提取出现的人物名 + 地名
  // 用于自动注入对应设定卡，保证 AI 上下文一致性
```

---

## 7. Tauri 后端 (src-tauri/)

### 7.1 角色定位

后端是**无状态命令服务层**：前端通过 `invoke()` 调用 `#[tauri::command]` 函数，后端操作 Store 并返回结果。AI 流式和测试连接使用**事件推送**。

### 7.2 AppData 状态

```rust
pub struct AppData {
    store: Mutex<Store>,           // 顺序化并发访问
    cancel: Mutex<Option<Arc<AtomicBool>>>,  // AI 取消信号
}
```

### 7.3 命令清单 (invoke_handler)

> 共 **15** 条命令（`7d8e980` 起：原 16 条，移除 `import_jsb`）。

| 命令 | 签名 | 返回 |
|------|------|------|
| **基础** | | |
| `init_info` | `()` | `{data_dir, key_fp, settings}` |
| `list_novels` | `()` | `Vec<NovelMeta>` |
| **小说与章节** | | |
| `load_novel` | `id: String` | `{novel, chapters:[{id,title,text}]}` |
| `create_novel` | `title,author,genre,desc` | `novel_id` |
| `save_novel` | `novel: Novel` | `()` |
| `save_chapter` | `novel_id, cid, text` | `{words, total_words, stats}` （增量字数） |
| `delete_chapter` | `novel_id, cid` | `()` |
| `delete_novel` | `id` | `()` |
| **设置** | | |
| `load_settings` / `save_settings` | | AppSettings |
| **导出** | | |
| `export_work` | `fmt, path, novel, chapters` | `()`（`fmt` 仅接受 `txt` / `md`，其他值返回"未知导出格式"） |
| `open_dir` | `path` | `()` (explorer/xdg-open) |
| **AI（已注册，但前端无调用方）** | | |
| `ai_start` | `cfg, messages` | `()` → 事件流 ai-chunk/done/error |
| `ai_cancel` | `()` | `()` |
| `ai_test` | `cfg` | `()` → 事件 ai-test-result |

### 7.4 AI 事件机制（保留，前端未接入）

```
前端 store.startAi()   ← ⚠️ 该函数与 AIPanel.vue 已随 AI 下线一并移除
  └─► invoke('ai_start')        （后端命令仍在，暂无人调用）
        └─► Rust 端启动线程
              ├─ ureq 流式请求
              ├─ app.emit('ai-chunk', text)  ──► 原前端 listen → aiStreamText += t
              ├─ app.emit('ai-done', ())     ──► 原前端 push 完整消息
              └─ app.emit('ai-error', msg)   ──► 原前端显示错误
```

> 原 `frontend/src/main.js` 中的 `listen("ai-chunk" / "ai-done" / "ai-error" / "ai-test-result")` 监听器已**全部移除**；如需重新接入，需先恢复面板与状态字段，再补回这四个监听器。

---

## 8. Vue 3 Web 前端 (frontend/)

### 8.1 技术选型

- **框架**：Vue 3.5 + `<script setup>` + Vite 6
- **状态**：`reactive()` 单例 store（无 Pinia）
- **通信**：@tauri-apps/api `invoke()` + `listen()`
- **编辑器**：CodeMirror 6
  - `@codemirror/commands`（编辑命令/选区）
  - `@codemirror/lang-markdown`（可选高亮）
  - `@codemirror/search`（查找替换）
  - `@codemirror/state` / `@codemirror/view`（自定义扩展）
- **对话框**：@tauri-apps/plugin-dialog（文件选择/保存）
- **打开外部**：@tauri-apps/plugin-opener

### 8.2 响应式状态 (store.js)

全局响应式状态（无 Pinia，单一 `reactive()` 单例）：

```js
store = reactive({
  ready, dataDir, keyFp, settings,
  library, novel, chapters,          // chapters = {cid: text}
  dirty, openTabs, activeTab,

  // ======== 布局 / 导航模型 ========
  ribbonTab,   // "writing" | "lore" | "tools" | "view"   Ribbon 4 个选项卡
  leftTree,    // "chapters" | "outline" | "characters" | "world" | "tasks"  左栏 5 棵树
  rightTool,   // null(工具格) | "namer" | "map" | "graph" | "timeline" | "tasks"
  timelineOpen,// 布尔：顶栏横向时间轴展开/收起（默认 false）
  mapPos,      // {locId: {x, y}} 地图节点位置
  fullView,    // null | "library" | "stats" | "settings" | "graph"  全屏视图
  leftWidth, rightWidth,   // 默认 300 / 400，可拖拽
  leftOpen, rightOpen,     // 左/右栏显示开关
  aiPanelOpen,             // ⚠️ 死字段：无消费方
  focusMode,

  paletteOpen, paletteQuery, findOpen, findReplace,

  selChar, selLoc, selEvent, selOutline, selChain, selTask, showRelCanvas, canvasPos,
  // ⚠️ selChain 固定为 "all"（任务链过滤实际用 LeftPane.vue 内局部状态）；
  //    showRelCanvas 无消费方

  dialog, toast, toastOk,      // ⚠️ toastOk 无消费方

  selectedText, cursorPos, wordCount, saveTimer, lastAutosave,
})
```

> 旧的 `store.activity`（活动栏导航）已随前端重构删除，导航改由 `ribbonTab` / `leftTree` / `rightTool` / `fullView` 四个字段共同表达。

### 8.3 业务动作 (store.js)

模块导出（`export`）共 **18 项**：

| 函数 | 作用 |
|------|------|
| `store` | 全局响应式状态单例 |
| `toast(msg, ok = true)` | 显示 3 秒浮层提示 |
| `today()` | 返回本地日期 `YYYY-MM-DD` |
| `openNovel(id)` | invoke `load_novel` → 填充 `chapters` → 更新最近打开 → 恢复上次章节（`settings.lastChapter[id]`），否则打开第一章；同时清空 `fullView` |
| `createNovel(form)` | invoke `create_novel` → 刷新书库 → 自动 `openNovel` |
| `closeNovel()` | `saveAll()` → 清空当前作品 → 回书库（`fullView = "library"`） |
| `openTab(cid)` | 标签页去重 + 置为活动页 + 记录 `settings.lastChapter` |
| `closeTab(cid)` | 先 `saveChapterNow(cid)` 落盘，再移除标签页与内存正文 |
| `markDirty(cid)` | 置脏 + 更新 `novel.meta.updated_at` + `scheduleSave()` |
| `scheduleSave()` | 2 秒 debounce → `saveAll()` |
| `saveChapterNow(cid)` | invoke `save_chapter` → 同步 `words` / `stats` / `total_words` |
| `saveAll()` | 遍历 dirty → 逐章保存 + invoke `save_novel` |
| `saveSettings()` | invoke `save_settings` |
| `applyTheme()` | 设置 `documentElement[data-theme]` + `--accent*` CSS 变量（由 rgb 计算 rgba 变体） |
| `allChapters(novel)` | 扁平化所有卷的章节 |
| `chapterTitle(cid)` | 按 cid 取标题，缺失回退「未命名」 |
| `countWords(s)` | 中文字符 + 英文单词 双语种统计（JS 版镜像 `util::count_words`） |
| `truncate(s, n)` | 超长截断加省略号 |

> 模块内私有函数 `touchRecent(id, title)`（写 `settings.recent` / `last_novel_id`）不导出。
>
> `startAi()` / `aiInsertToEditor()` / `aiCancel()` 已随 AI 下线删除。

### 8.4 组件装配 (App.vue)

```text
<App>                               根节点
  ├─ Ribbon                         顶部两行（选项卡 + 窗口控制 + 当前组按钮 + 时间轴折叠钮）
  ├─ 全屏视图 (store.fullView)
  │    ├─ .fv-bar                   「← 返回写作台」+ 视图名 + 《书名》
  │    └─ .fv-body
  │         ├─ Library              fullView = "library"
  │         ├─ StatsView            fullView = "stats"
  │         ├─ SettingsView         fullView = "settings"
  │         └─ RelationGraph        fullView = "graph"
  ├─ 写作台 (store.novel 存在且 fullView 为 null)
  │    ├─ TimelineTrack             v-if="!focusMode"（顶栏剪辑式横向时间轴）
  │    └─ .body (flex 横排)
  │         ├─ LeftPane             v-if="leftOpen && !focusMode"
  │         ├─ DragHandle           左分隔条（min 220 / max 560 / fallback 300）
  │         ├─ main.central
  │         │    ├─ EditorView      v-if="activeTab"
  │         │    └─ Empty 空态       「还没有打开章节」
  │         ├─ DragHandle           右分隔条（min 280 / max 640 / fallback 400）
  │         └─ RightPane            v-if="rightOpen && !focusMode"
  ├─ 无作品分支                      无 novel 时只渲染 <Library />（空书库欢迎页）
  ├─ StatusBar                      底部状态栏
  ├─ Palette                        v-if="paletteOpen"
  ├─ Modal                          v-if="dialog"
  ├─ ContextMenu                    常驻监听 jinshu:contextmenu
  └─ toast                          v-if="toast"
```

> 左右栏宽度由 `DragHandle.vue` 统一管理：拖动改宽度、**双击复位**、`saveSettings()` 落盘。
> 专注模式下 `TimelineTrack` / `LeftPane` / `RightPane` 一并隐藏，只留中央正文。

#### 左栏 5 棵树 (LeftPane.vue)

| Tab id | 名称 | 结构 |
|--------|------|------|
| `chapters` | 章节 | 卷 / 章 两级 |
| `outline` | 大纲 | 卷 / 章 / 节 / 要点 递归 |
| `characters` | 人物 | 按 `role` 分组，标「本」= 本章正文出现 |
| `world` | 世界 | 按 `kind` 分组，支持嵌套 |
| `tasks` | 任务线 | 按角色分组 + 任务链过滤，状态 ○/◐/●，标「暗」= 暗线 |

#### 右栏 5 个工具 (RightPane.vue)

均为**右栏内嵌面板**（非弹窗），由 `store.rightTool` 切换：

| tool id | 名称 | 内容 |
|---------|------|------|
| `namer` | 起名机 | 类型/风格/性别/数量 + 生成列表，可复制 / 插入正文 |
| `map` | 地图 | SVG 可拖拽地点节点画布 + 就地编辑 |
| `graph` | 关系网 | 人物关系画布 + 重置布局 |
| `timeline` | 时间轴 | 事件列表就地编辑 |
| `tasks` | 任务线 | 三列看板，卡片拖拽换列，归属角色 / 关联章节 / 明暗线 |

`rightTool` 为 `null` 时显示工具格（上述 5 个工具的入口卡片）。

#### StatusBar

- 左：`🔒 加密存储` ｜ 书名 ｜ `总字数 N` ｜ `今日 +N` ｜ `本章 N` ｜ `行 X · 列 Y`
- 右：`UTF-8` ｜ `🔑 密钥指纹` ｜ `📂 打开数据目录`按钮

### 8.5 主题系统 (styles/theme.css)

CSS 变量双主题 + 运行时强调色注入：

```css
:root { --chrome: --panel: --editor: --text: --accent: --ok: --danger: ... }
[data-theme="dark"]  { 深色调色板 }
[data-theme="light"] { 浅色调色板 }

/* store.applyTheme() 动态设置 accent 系列：
   --accent / --accent-soft / --accent-softer / --accent-strong
   通过 rgb(r,g,b) 计算 rgba 变体
*/
```

### 8.6 CodeMirror 6 编辑器 (components/EditorView.vue)

核心特性：
- 初始化时安装 editorSettings 对应扩展（字体/行高/换行/行号/两端对齐/每行字数/搜索/Markdown高亮）
- `Prec.highest` keymap：`Mod-f` 打开查找、`Mod-h` 打开查找替换，另挂载 `defaultKeymap` / `historyKeymap` / `indentWithTab`
- `view.dispatch({ changes: { insert } })` 响应 `jinshu:insert` 自定义事件
- `updateListener` 双向同步：Vue store.chapters[cid] ⇄ CM6 文档
- 选区变化 → `store.selectedText`
- 光标变化 → `store.cursorPos`

---

## 9. 导出模块

> 路径：[src-tauri/src/export.rs](file:///c:/Code/JinShu-rust/src-tauri/src/export.rs)
>
> 仅 `export_txt` / `export_md` 两个函数，**无任何导入能力**（`export_jsb` / `import_jsb` / `import_to_store` / `default_export_dir` 已随 `.jsb` 备份一并移除，`scrypt`、`flate2` 依赖同步删除）。

### 9.1 txt 纯文本
```
《书名》
作者：xxx
简介：xxx

====================
第一卷
====================

第一章 标题
------------
正文……
```

### 9.2 md Markdown
```md
# 书名

> 作者：xxx

简介

---

## 第一卷

### 第一章 标题

正文……
```

---

## 10. 工具函数 (util.rs)

> 路径：[src-tauri/src/util.rs](file:///c:/Code/JinShu-rust/src-tauri/src/util.rs)

### 10.1 util.rs 工具

```rust
pub fn new_id() -> String                              // UUID v4 simple
pub fn today() -> String                               // YYYY-MM-DD
pub fn days_ago(n: i64) -> String                      // 统计回溯用
pub fn now_ts() -> i64                                 // Unix 秒时间戳
pub fn format_ts(ts: i64) -> String                    // YYYY-MM-DD HH:MM
pub fn count_words(s: &str) -> u64                     // CJK逐字 + 英文按词
pub fn count_chars(s: &str) -> u64                     // 非空白字符总数
pub fn truncate_chars(s: &str, max: usize) -> String   // AI 上下文裁剪
pub fn tail_chars(s: &str, max: usize) -> String       // 取末尾若干字符
pub fn indent_two(s: &str) -> String                   // 首段加两字全角空格
```

---

## 11. 模块依赖关系图

### 11.1 依赖方向

```
frontend/*.vue + store.js
  └─► api.js (invoke 封装，12 个方法)
        └──► IPC ──► src-tauri/src/main.rs (#[tauri::command]，15 条)
                              ├─► store.rs
                              │     └─► crypto.rs
                              ├─► model.rs
                              ├─► ai_client.rs  ⚠️ 前端已无调用方
                              │     └─► ai_prompts.rs
                              ├─► export.rs   (仅 txt / md)
                              └─► util.rs
```

**无循环依赖保证**：`model.rs` 只依赖 `util.rs`（ID + 时间）；`store.rs` 依赖 `model.rs` + `crypto.rs`；`main.rs` 命令层位于以上模块之上。

### 11.2 外部 crates 依赖说明

| Crate | 用途 |
|-------|------|
| `tauri 2` | WebView 桌面应用框架 |
| `serde + serde_json` | 所有数据模型序列化 |
| `aes-gcm 0.10` | AES-256-GCM AEAD 加解密（RustCrypto 项目） |
| `sha2 0.10` | 密钥指纹 (SHA-256) |
| `rand 0.8` | 密钥/nonce CSPRNG |
| `ureq 2` | 同步阻塞 HTTP 客户端（用于 AI SSE；仍依赖，但无调用方） |
| `chrono 0.4` | 时间戳/日期格式化 |
| `uuid 1 (v4)` | 所有实体 ID 生成 |
| `vue 3.5` / `vite 6` | 前端框架与构建 |
| `@codemirror/* 6.x` | 编辑器 |
| `@tauri-apps/* 2.x` | Tauri 前端 API + CLI |

---

## 12. 核心数据流

### 12.1 编辑器输入 → 磁盘

```
用户输入字符
   │
   ▼
EditorView (CodeMirror 6)
   │  文档变更
   ▼
store.markDirty(cid)
   │  dirty.insert(cid); cmeta.updated_at = now
   ▼
store.scheduleSave()（2 秒 debounce）→ saveAll()
   │
   ▼
invoke('save_chapter')
   │  1. util::count_words(text) → 新字数
   │  2. today_stats += delta (新增部分)
   │  3. novel.meta.total_words = sum
   │  4. Store::save_chapter → crypto::encrypt_file(cid.jsr)
   │  5. Store::save_novel   → crypto::encrypt_file(novel.jsr)
   │  6. dirty.remove(cid)
   ▼
完成（加密文件落盘）
```

### 12.2 启动流程

```
npm run build (前端) → frontend/dist/
       │
       ▼
tauri build 编译 Rust 后端
   ├─ tauri-build::build.rs 嵌入 dist 到二进制
   └─► jinshu.exe
         │
         ▼
用户双击 exe
   ├─ Store::init() → 解析 data_dir + 生成/加载 .jinshu_key
   ├─ tauri::Builder::manage(AppData{store,cancel})
   ├─ 注册 15 条 commands（含 3 条 AI 命令，前端未调用）
   └─ 启动 WebView → 加载 index.html
         │
         ▼
   Vue main.js
      ├─ watch(settings.theme / accent / ui_scale) → applyTheme()
      ├─ window keydown 监听（全局快捷键，见 §16）
      ├─ init()：
      │    ├─ api.initInfo() → {data_dir, key_fp, settings}
      │    ├─ store.dataDir / keyFp / settings = 结果
      │    ├─ api.listNovels() → store.library
      │    ├─ applyTheme() → 设置 CSS 变量
      │    ├─ store.ready = true
      │    └─ settings.last_novel_id 存在 → openNovel(id)（失败则回退刷新书库）
      └─ createApp(App).mount("#app")
            │
            ▼
         App.vue（ready 后渲染）
            ├─ fullView 为空 → 写作台（Ribbon + TimelineTrack + 三栏 + StatusBar）
            └─ 无作品 → 只渲染 Library 欢迎页
```

---

## 13. 构建与运行方式

### 13.1 从源码构建

```bash
# 前置：Rust 1.85+ 、Node 20+
# Windows：WebView2（Win10/11 自带）
# Linux：webkit2gtk-4.1 + gtk3 + noto-fonts-cjk

# 1. 前端构建
cd frontend
npm install
npm run build     # → frontend/dist/
cd ..

# 2. Tauri 后端构建
cd src-tauri
cargo build --release

# 产物位置
./src-tauri/target/release/jinshu        # Linux
./src-tauri/target/release/jinshu.exe    # Windows
```

### 13.2 发布打包

#### Windows 便携版
```bash
bash build_pkg_win.sh
# 产物：dist/JinShu-rust-win64-tauri.zip (已内嵌前端 + 可运行)
```

#### Arch Linux AUR 包
```bash
cd packaging/arch
makepkg -si
# 系统级安装，默认 /opt/jinshu-rust
# 启动前设置 JINSHU_DATA_DIR=~/jinshu-data 或 sudo chown -R $USER /opt/jinshu-rust
```

### 13.3 开发调试

```bash
# 前端热更新
cd frontend && npm run dev    # Vite dev server @ http://localhost:5173
# 另开终端：cd src-tauri && cargo run (需指向 dev 前端，见 tauri.conf.json devUrl)
```

### 13.4 运行测试

```bash
# 共 6 项
cd src-tauri && cargo test
#   - crypto_roundtrip         AES-256-GCM 加密往返 + 错误密钥必须失败 + AAD 绑定
#   - store_encrypted_files    数据目录全盘加密验证（无明文文件）
#   - word_count_cjk           CJK/英文混排字数统计正确性
#   - novel_chapter_flow       卷/章 CRUD 业务
#   - export_formats           txt / md 导出格式
#   - outline_parse            大纲缩进文本解析
```

---

## 14. 配置与环境变量

| 环境变量 | 作用 |
|----------|------|
| `JINSHU_DATA_DIR` | 强制指定数据目录，覆盖"exe同目录/data"默认行为。系统级安装时必须设置用户可写路径。 |

---

## 15. 测试覆盖

### 15.1 测试用例（6 项）

| 测试文件 | 测试名 | 验证点 |
|----------|--------|--------|
| `src-tauri/src/model.rs` | `word_count_cjk` | 汉字/标点/英文/空格混合统计边界 |
| `src-tauri/src/model.rs` | `novel_chapter_flow` | 新建卷→加章→查询→删除→计数同步 |
| `src-tauri/src/crypto.rs` | `crypto_roundtrip` | AES-GCM 加解对称 + 密钥错误/AAD 错误必失败 |
| `src-tauri/src/store.rs` | `store_encrypted_files` | 1.无明文泄漏（字节扫描）；2.无 `.jsr` / `.jinshu_key` 外的扩展名；3.读回解密正确 |
| `src-tauri/src/export.rs` | `export_formats` | txt / md 两种导出格式的结构与内容 |
| `src-tauri/src/ai_prompts.rs` | `outline_parse` | 大纲缩进文本 → 树结构的解析（AI 未接入，测试仍保留） |

**加密落盘测试的严格性**：遍历 `data_dir` 所有文件扩展名必须为 `jsr` 或名称为 `.jinshu_key`，保证不产生临时泄漏文件。

---

## 16. 快捷键与命令系统

### 16.1 命令面板 (Ctrl+P)

`components/Palette.vue` 中实现模糊匹配（按标签子串包含）+ 键盘方向键选择 + Enter 执行，共 **13 条命令**：

| # | 命令 | 展示快捷键 | 实际动作 |
|---|------|-----------|----------|
| 1 | 新建小说 | Ctrl+N | 打开 `dialog = {kind:"newNovel"}` |
| 2 | 打开书库 | Ctrl+O | `fullView = "library"` + 刷新书库 |
| 3 | 保存全部 | Ctrl+S | `saveAll()` + toast |
| 4 | 导出作品 | — | 打开导出弹窗（无作品时提示） |
| 5 | 新建章节 | — | 打开新建章节弹窗 |
| 6 | 新建分卷 | — | 打开新建分卷弹窗 |
| 7 | 本地起名机（人物/书名/地名） | — | 打开起名机弹窗 |
| 8 | 切换左栏 | Ctrl+B | `leftOpen = !leftOpen` |
| 9 | 切换深色/浅色主题 | — | 切换 `settings.theme` 并落盘 |
| 10 | 查找 | Ctrl+F | `findOpen = true` |
| 11 | 写作统计 | — | `fullView = "stats"` |
| 12 | 打开设置 | — | `fullView = "settings"` |
| 13 | 关于锦书 | — | 打开关于弹窗 |

> 前端的命令面板**无任何 AI 命令**（AI 条目已随 AI 下线移除）。
>
> ⚠️ 表中「展示快捷键」列是面板里渲染的提示文字，**不保证等于真实绑定**：`Ctrl+O` 无任何绑定，`Ctrl+F` 在全局也无绑定（真正的查找键是编辑器内的 `Mod-f`）。

### 16.2 命令与快捷键映射表（真实绑定）

| 快捷键 | 作用 | 绑定位置 | 备注 |
|--------|------|----------|------|
| `Ctrl+S` | 保存全部 | `main.js` 全局 keydown | 无条件生效 |
| `Ctrl+P` | 切换命令面板 | `main.js` 全局 keydown | **焦点不在编辑器内**才生效 |
| `Ctrl+B` | 切换左栏显示/隐藏 | `main.js` 全局 keydown | 编辑器内同样生效 |
| `Ctrl+=` | 正文字号 +1（上限 32） | `main.js` 全局 keydown | ⚠️ 未调用 `saveSettings()`，不落盘 |
| `Ctrl+-` | 正文字号 −1（下限 10） | `main.js` 全局 keydown | ⚠️ 未调用 `saveSettings()`，不落盘 |
| `Ctrl+N` | 新建小说（打开弹窗） | `main.js` 全局 keydown | **焦点不在编辑器内**才生效 |
| `Esc` | 关闭命令面板 / 查找条 / 弹窗 | `main.js` 全局 keydown | — |
| `Mod-f` | 查找 | CodeMirror keymap（`Prec.highest`） | 仅编辑器内 |
| `Mod-h` | 查找替换 | CodeMirror keymap（`Prec.highest`） | 仅编辑器内 |
| `Ctrl+滚轮` | 横向时间轴轨道缩放 | `TimelineTrack.vue` | 仅时间轴区域内 |

> 编辑器内另挂载 `defaultKeymap`（撤销/重做/光标移动等标准编辑键）、`historyKeymap`、`indentWithTab`。
>
> **无按键绑定的键**：`Ctrl+O`（仅作为命令面板展示标签存在，与实际不符）、`Ctrl+Shift+F`、`Ctrl+W`、`Ctrl+J`。
> 其余功能（新建章节/分卷、导出、写作统计、设置、关于、起名机、关系网、地图、任务线等）均**无快捷键**，只能经 Ribbon 按钮、左/右栏或命令面板触发。

### 16.3 Ribbon 4 组按钮

`components/Ribbon.vue` 顶部第一行为选项卡（`writing | lore | tools | view`），第二行为窗口控制、当前组按钮与时间轴折叠钮。

| 组 | 按钮 |
|----|------|
| **写作** (`writing`) | 章节（新建/新建卷/重命名/删除）、排版（首行缩进/两端对齐/增大字号/减小字号）、文件（保存/导出） |
| **设定** (`lore`) | 人物（新建人物/关系网）、世界观（新建地点/新建事件）、大纲（从章节生成/新建节点）、任务（新建任务/新建任务链） |
| **工具** (`tools`) | 查找（查找替换/全书搜索）、起名（起名机）、数据（书库/统计/数据目录/设置） |
| **视图** (`view`) | 左栏（章节/大纲/人物/世界树、隐藏·显示左栏）、顶栏横向轴（展开·收起时间轴）、右栏（工具格/起名机/地图/关系网/时间轴/任务、隐藏·显示右栏）、窗口（专注写作/深浅主题/正文设置） |

---

## 附录：安全边界说明

> 摘自 README 并补充实现细节

1. **密钥文件**：`data/.jinshu_key` 32字节纯随机，Unix 权限 0o600。删除此文件后旧数据**无法解密**，请定期整目录备份。
2. **AAD 绑定**：加密时使用文件名作为附加认证数据，防止攻击者将"章A密文"重命名为"章B密文"后被解密为章B的正文冒充。
3. **原子写入**：所有文件先写 `*.tmp` 再 rename，保证断电崩溃不损坏原文件（只可能丢失最近一次）。
4. **不污染系统**：不写 `%APPDATA%`、`%TEMP%`、`~/.local`、`/tmp`（已验证：所有持久文件扩展名均为 `.jsr` 或名为 `.jinshu_key`）。
5. **威胁模型**：
   - 可防：随手读盘、同事借用、备份盘丢失、云盘同步时的明文泄漏。
   - 不可防：同权限进程注入读内存、内存dump、物理机调试类攻击（密钥与数据同机）。
   - 高强度方案：定期整目录备份（含 `.jinshu_key`）；`.jsb` 密码备份能力已移除，如需更强保护需另行设计。

---

## 附录 B：已知问题（自查所得，均未修复）

> 以下问题来自 2026-09-16 前端重构后的代码通读，如实记录，未作美化。

| # | 位置 | 问题 |
|---|------|------|
| 1 | `views/Library.vue` | 欢迎页仍在写 `store.activity = 'library'` —— `activity` 字段已被删除，属重构遗留的**悬空引用**（当前无害，但会污染 store 对象）。 |
| 2 | `components/Ribbon.vue` | 「工具 → 查找」组里的**「全书搜索」按钮实际打开的是起名机**（`store.rightTool = "namer"`），疑似写错。 |
| 3 | `store.js` | `aiPanelOpen`、`showRelCanvas`、`toastOk` 三个字段**无任何消费方**（死字段）；`selChain` 固定为 `"all"` 且无人修改，任务链过滤实际使用 `LeftPane.vue` 内的局部状态。 |
| 4 | `main.js` | `Ctrl+=` / `Ctrl+-` 调整正文字号后**没有调用 `saveSettings()`**，字号变更不落盘，重启即丢失。 |
| 5 | `LeftPane.vue` / `store.js` | 注释仍写「4 个树 Tab」（`leftTree` 注释也只列了 `chapters/outline/characters/world`），实际已是 **5 个**（含 `tasks`）。 |

---

*本 Code Wiki 对应代码版本：仓库 HEAD（Tauri 单引擎，2026-09-16）*
