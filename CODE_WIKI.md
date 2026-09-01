# 锦书 JinShu-rust · Code Wiki

> 为中文小说创作而生的本地编辑器，Rust + Tauri 2（Vue 3 前端）双栈架构，全部数据以 AES-256-GCM 加密缓存于安装目录。

---

## 目录

1. [项目总览与技术栈](#1-项目总览与技术栈)
2. [整体架构设计](#2-整体架构设计)
3. [目录结构说明](#3-目录结构说明)
4. [数据模型层 (Model)](#4-数据模型层-model)
5. [加密存储层 (Store / Crypto)](#5-加密存储层-store--crypto)
6. [AI 服务层 (AI)](#6-ai-服务层-ai)
7. [egui 原生前端 (src/)](#7-egui-原生前端-src)
8. [Tauri 后端 (src-tauri/)](#8-tauri-后端-src-tauri)
9. [Vue 3 Web 前端 (frontend/)](#9-vue-3-web-前端-frontend)
10. [导出 / 导入模块](#10-导出--导入模块)
11. [工具函数与通用组件](#11-工具函数与通用组件)
12. [模块依赖关系图](#12-模块依赖关系图)
13. [核心数据流](#13-核心数据流)
14. [构建与运行方式](#14-构建与运行方式)
15. [配置与环境变量](#15-配置与环境变量)
16. [测试覆盖](#16-测试覆盖)
17. [快捷键与命令系统](#17-快捷键与命令系统)

---

## 1. 项目总览与技术栈

### 1.1 项目定位

**锦书 (JinShu-rust)** 是一款面向中文小说创作者的本地 IDE 风格编辑器，核心特性：

| 能力 | 实现方式 |
|------|----------|
| 章节 / 分卷管理 | 卷→章 树形结构，多标签页，拖拽排序 |
| 写作编辑器 | egui 自研 / CodeMirror 6（Tauri版） |
| 大纲系统 | 卷→章→节→要点 四级树形大纲 |
| 人物设定 | 人物卡 + 关系网画布（可拖拽） |
| 世界观 | 地点层级树（国家→城市→建筑） |
| 时间线 | 事件排序 + 关联章节/人物/地点 |
| 任务看板 | 任务链 + 三列看板（待办/进行中/已完成） |
| 写作统计 | 总字数 / 今日 / 连续天数 / 30天趋势 |
| AI 创作助手 | 续写/润色/扩写/大纲/逻辑检查/一致性检查等14种动作 |
| Lorebook 注入 | 正文出现人物/地名时自动注入设定卡 |
| 数据安全 | AES-256-GCM 加密落盘，不写系统目录 |
| 导出格式 | txt / md / .jsb（Scrypt 密码加密备份） |

### 1.2 技术栈全景

```
┌─────────────────────────────────────────────────────────────────┐
│                        用户界面层                                │
│  ┌──────────────────────┐   ┌───────────────────────────────┐   │
│  │  egui 0.36 原生GUI    │   │  Tauri 2 + Vue 3 + Vite 6     │   │
│  │  (src/ 目录)          │   │  (src-tauri/ + frontend/)    │   │
│  │  自绘编辑器+控件      │   │  CodeMirror 6 编辑器         │   │
│  └──────────┬───────────┘   └──────────────┬────────────────┘   │
│             │                              │                    │
└─────────────┼──────────────────────────────┼────────────────────┘
              │                              │
┌─────────────▼──────────────────────────────▼────────────────────┐
│                      业务逻辑层（Rust）                          │
│  ┌───────────┐  ┌───────────┐  ┌───────────┐  ┌───────────┐     │
│  │   Model   │  │   Store   │  │    AI     │  │  Export   │     │
│  │  数据模型  │  │  加密存储  │  │  流式客户端│  │  导入导出  │     │
│  └───────────┘  └───────────┘  └───────────┘  └───────────┘     │
└─────────────┬──────────────────────────────┬────────────────────┘
              │                              │
┌─────────────▼──────────────────────────────▼────────────────────┐
│                       基础设施层                                 │
│  AES-256-GCM (aes-gcm)  |  Scrypt (scrypt)  |  Zlib (flate2)    │
│  ureq (HTTP)  |  UUID v4  |  Chrono  |  Serde JSON              │
│  rand (CSPRNG) |  SHA-256 (密钥指纹)                            │
└─────────────────────────────────────────────────────────────────┘
```

### 1.3 双前端说明

本项目维护**两套独立的前端实现**，共享底层业务逻辑：

| 版本 | 路径 | UI框架 | 适用场景 |
|------|------|--------|----------|
| egui 原生版 | `src/` | eframe/egui 0.36 | 轻量、极致小体积、无浏览器依赖 |
| Tauri Web版 | `src-tauri/` + `frontend/` | Tauri 2 + Vue 3 | 现代化IDE体验、CodeMirror编辑器、官方推荐发布版本 |

---

## 2. 整体架构设计

### 2.1 分层架构

```
┌───────────────────────────────────────────────────────┐
│                    Presentation (UI)                   │
│  egui widgets ─── shell/App ─── AppState              │  ← src/ 原生
│  Vue Components ─ store.js ── Tauri invoke            │  ← frontend/
├───────────────────────────────────────────────────────┤
│                   Application Layer                    │
│  AppState (小说生命周期 / 标签页 / AI 调度 / 保存)     │
│  Tauri Commands (命令接口层，前端 ↔ 后端桥接)          │
├───────────────────────────────────────────────────────┤
│                     Domain Layer                       │
│  Novel · Volume · ChapterMeta · OutlineNode           │
│  Character · Location · TimelineEvent · Task          │
│  业务方法: add_chapter / delete_chapter / total_words  │
├───────────────────────────────────────────────────────┤
│                  Infrastructure Layer                  │
│  Store (加密IO) · Crypto (AES/Scrypt)                 │
│  AI Client (流式HTTP) · Export (txt/md/jsb)           │
│  Util (UUID/时间/字数统计)                             │
└───────────────────────────────────────────────────────┘
```

### 2.2 设计原则

1. **数据本地优先**：所有数据加密切块存在安装目录，不写 `%APPDATA%`/`~/.local`/临时目录
2. **密钥与数据解耦**：本地密钥 `.jinshu_key` + 可选密码备份 `.jsb`（Scrypt 派生密钥）
3. **无状态AI客户端**：流式 SSE 输出，通过 mpsc channel / Tauri events 推送增量
4. **内存态 + 定期刷盘**：正文在内存 `HashMap<cid, String>`，脏位标记 + 自动保存（默认5秒）
5. **前端自由**：egui 与 Tauri 版共享 model/store/ai/crypto/export 核心代码

---

## 3. 目录结构说明

```
JinShu-rust/
├── assets/                     # 静态资源（嵌入二进制）
│   ├── fonts/                  #   NotoSansSC / NotoSerifSC 可变字体
│   └── icons/                  #   app.ico / app.jpg
├── frontend/                   # Tauri 版 Vue 3 前端（Vite 构建）
│   ├── src/
│   │   ├── components/         #   UI 组件
│   │   │   ├── TitleBar.vue        #   自定义标题栏 + 窗口控制
│   │   │   ├── ActivityBar.vue     #   左侧图标活动栏
│   │   │   ├── SidePanel.vue       #   侧边面板（章节/大纲/人物…）
│   │   │   ├── EditorView.vue      #   CodeMirror 6 编辑器视图
│   │   │   ├── AIPanel.vue         #   AI 助手对话面板
│   │   │   ├── Palette.vue         #   Ctrl+P 命令面板
│   │   │   ├── Modal.vue           #   对话框（新建/导出/删除…）
│   │   │   ├── ContextMenu.vue     #   右键菜单
│   │   │   ├── StatusBar.vue       #   底部状态栏
│   │   │   └── Icon.vue            #   图标组件
│   │   ├── views/              #   全屏活动视图
│   │   │   ├── Library.vue         #   书库列表
│   │   │   ├── StatsView.vue       #   写作统计
│   │   │   ├── SettingsView.vue    #   设置页
│   │   │   └── DetailViews.vue     #   人物/地点/时间线/任务/大纲详情
│   │   ├── store.js            #   全局响应式状态 + 业务动作
│   │   ├── api.js              #   Tauri invoke 命令封装
│   │   ├── names.js            #   角色定位/地点类型/任务状态常量
│   │   ├── prompts.js          #   AI 动作提示词模板（前端版）
│   │   ├── App.vue             #   根组件：布局装配
│   │   ├── main.js             #   Vue 入口 + 主题初始化
│   │   └── styles/theme.css    #   CSS 变量主题系统（深浅色+8种强调色）
│   ├── index.html
│   ├── package.json            #   Vue 3.5 + Vite 6 + CodeMirror 6
│   └── vite.config.js
│
├── src/                        # egui 原生版本（Rust GUI）
│   ├── main.rs                 #   eframe 入口：窗口配置 + 启动
│   ├── app.rs                  #   AppState 核心状态 + 小说/章节/AI 业务方法
│   ├── shell.rs                #   eframe App impl：布局装配 + 快捷键 + 命令面板
│   ├── model.rs                #   领域模型：Novel/Volume/Character/… 及业务方法
│   ├── editor.rs               #   自研编辑器组件（行号/高亮/光标跳转）
│   ├── ai_panel.rs             #   AI 助手面板 UI + 动作消息构造
│   ├── settings.rs             #   设置页 UI
│   ├── dialogs.rs              #   模态对话框（新建/导出/导入/关于）
│   ├── export.rs               #   txt/md/jsb 导出 + jsb 导入
│   ├── theme.rs                #   Palette 调色板 + CJK 字体安装
│   ├── util.rs                 #   UUID / 时间 / 字数统计 / 文本裁剪
│   ├── widgets.rs              #   通用控件（activity_btn / icon_btn / empty_state…）
│   ├── ai/
│   │   ├── mod.rs
│   │   ├── client.rs           #   OpenAI+Anthropic SSE 流式客户端（ureq）
│   │   └── prompts.rs          #   AI 提示词模板 + Lorebook 注入逻辑
│   ├── store/
│   │   ├── mod.rs              #   Store 结构体 + 设置/小说/章节 IO
│   │   └── crypto.rs           #   AES-256-GCM + Scrypt + JSB 格式
│   └── views/
│       ├── mod.rs              #   欢迎页 / 书库 / 搜索 / 统计视图
│       ├── chapters.rs         #   章节树 + 大纲树 UI
│       └── world.rs            #   人物 + 地点 + 时间线 + 任务 UI
│
├── src-tauri/                  # Tauri 版 Rust 后端
│   ├── src/
│   │   ├── main.rs             #   Tauri Builder + 所有 #[tauri::command]
│   │   ├── model.rs            #   ≈ src/model.rs （Tauri 版数据模型）
│   │   ├── store.rs            #   ≈ src/store/mod.rs
│   │   ├── crypto.rs           #   ≈ src/store/crypto.rs
│   │   ├── ai_client.rs        #   ≈ src/ai/client.rs
│   │   ├── ai_prompts.rs       #   ≈ src/ai/prompts.rs
│   │   ├── export.rs           #   ≈ src/export.rs
│   │   └── util.rs             #   ≈ src/util.rs
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
├── Cargo.toml                  #   egui 版根 crate（jinshu-rust）
├── build.rs                    #   winresource Windows 资源编译
├── build_pkg_win.sh            #   Windows 便携版打包脚本
├── build_pkg_arch.sh           #   Arch Linux 便携版打包脚本
├── LICENSE (MIT)
└── README.md
```

---

## 4. 数据模型层 (Model)

> 路径：[src/model.rs](file:///c:/Code/JinShu-rust/src/model.rs) / [src-tauri/src/model.rs](file:///c:/Code/JinShu-rust/src-tauri/src/model.rs)

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
}
```

#### Task / TaskChain — 任务看板

- Task.status: 0=待办, 1=进行中, 2=已完成
- Task.chain_id 关联 TaskChain；未关联的归入 "无分组"

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

> 路径：[src/store/mod.rs](file:///c:/Code/JinShu-rust/src/store/mod.rs) + [src/store/crypto.rs](file:///c:/Code/JinShu-rust/src/store/crypto.rs)

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

- `.jsr` = JinShu Rust 本地加密格式
- `.jsb` = JinShu Backup 跨设备密码备份格式

### 5.3 加密方案

#### 5.3.1 JSR 本地格式（AES-256-GCM）

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

#### 5.3.2 JSB 密码备份格式（Scrypt + AES-256-GCM + Zlib）

```
文件布局: [MAGIC 4B][VER 1B][SALT 16B][NONCE 12B][CIPHERTEXT(zlib(json))...]
          J S B 1    0x01    Scrypt盐   AES随机数
```

- **密钥派生**：`Scrypt(password, salt, N=2^15, r=8, p=1, dkLen=32)`
- **压缩**：Zlib default 压缩后加密
- **用途**：跨设备迁移、云盘备份（密码不写磁盘）

核心函数：
```rust
pub fn encrypt_jsb(password: &str, plain: &[u8]) -> Result<Vec<u8>, String>
pub fn decrypt_jsb(password: &str, data: &[u8]) -> Result<Vec<u8>, String>
pub fn derive_key_from_password(password: &str, salt: &[u8]) -> Result<[u8;32], String>
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
├── nav_expanded: Option<bool>   # （Tauri版）导航栏收起/展开
├── recent: Vec<RecentNovel>     # 最近打开（最多8项）
├── editor: EditorSettings
│   ├── font: "serif" | "sans"
│   ├── font_size: f32           # 默认 17
│   ├── line_spacing: f32        # 默认 1.9
│   ├── wrap: bool
│   ├── markdown_highlight: bool # 中文写作默认关闭
│   ├── auto_indent: bool        # 首段自动两字缩进
│   └── show_line_numbers: bool
└── ai: AiSettings
    ├── protocol: "openai" | "anthropic"
    ├── base_url
    ├── api_key                 # 加密存储
    ├── model                   # 如 deepseek-chat / claude-sonnet-4-5
    ├── temperature             # 默认 0.8
    ├── max_tokens              # 默认 4096
    ├── system_prompt           # 默认：资深中文小说创作助手…
    ├── timeout_secs            # 默认 180
    └── inject_lore: bool       # 自动 Lorebook 注入开关
```

---

## 6. AI 服务层 (AI)

> 路径：[src/ai/client.rs](file:///c:/Code/JinShu-rust/src/ai/client.rs) + [src/ai/prompts.rs](file:///c:/Code/JinShu-rust/src/ai/prompts.rs)

### 6.1 架构：独立线程 + SSE 流式 + Channel

```
UI Thread (AppState.poll_ai)
      │
      │  mpsc::Receiver<AiEvent>
      │
      ▼
Worker Thread (ureq Agent)
  ├── stream_openai()    ──► POST /chat/completions ?stream=true
  └── stream_anthropic() ──► POST /v1/messages ?stream=true
      │
      └── 按行解析 SSE → AiEvent::Chunk(text) 回传
          完成 → AiEvent::Done
          错误 → AiEvent::Error(msg)
```

egui 版通过 `mpsc::channel` + 每帧 `poll_ai()` 轮询；Tauri 版通过 `app.emit()` 推送前端事件：
- `ai-chunk` / `ai-done` / `ai-error` / `ai-test-result`

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

Tauri 版额外命令：
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

## 7. egui 原生前端 (src/)

### 7.1 AppState — 全局状态核心

> 路径：[src/app.rs](file:///c:/Code/JinShu-rust/src/app.rs)

```rust
pub struct AppState {
    // 存储与设置
    pub store: Option<Store>,
    pub settings: AppSettings,
    pub pal: Palette,
    pub init_error: Option<String>,

    // 书籍数据
    pub library: Vec<NovelMeta>,
    pub novel: Option<Novel>,
    pub chapters: HashMap<String, String>,   // 内存态正文（cid → 全文）
    pub dirty: HashSet<String>,              // 脏章标记
    pub open_tabs: Vec<String>,              // 标签顺序
    pub active_tab: Option<String>,
    pub last_cursor: HashMap<String, usize>, // cid → 光标字符位置
    pub selected_text: String,

    // UI 状态
    pub activity: Activity,                  // 当前活动视图
    pub sidebar_open / ai_panel_open / focus_mode: bool,
    pub find: Option<FindState>,             // 查找替换条
    pub palette_open / palette_query: String,

    // AI 状态
    pub ai_msgs: Vec<AiMsg>,
    pub ai_streaming: bool,
    pub ai_stream_text: String,
    pub ai_cancel: Arc<AtomicBool>,
    pub ai_rx: Option<Receiver<AiEvent>>,
    pub summaries: HashMap<String, String>,  // cid → AI 摘要缓存

    // 对话框表单
    pub dialog: Option<DialogKind>,
    pub form_title/author/genre/desc: String,
    pub export_fmt: String,
    pub jsb_pwd/jsb_pwd2: String,

    // 画布与选择
    pub canvas_pos: HashMap<String, egui::Pos2>,  // 关系网节点位置
    pub sel_char/sel_loc/sel_event/sel_outline/sel_chain,
}
```

**Activity 枚举**（10 个活动视图）：Library, Chapters, Outline, Characters, World, Timeline, Tasks, Search, Stats, Settings

### 7.2 关键业务方法

| 方法 | 说明 |
|------|------|
| `open_novel(id)` | 加载 Novel + 全部章节到内存；恢复自动打开 |
| `close_novel()` | 保存所有 + 清空 + 回退书库 |
| `open_tab(cid)` / `close_tab(cid)` | 标签页管理；关闭时自动保存 |
| `mark_dirty(cid)` | 标记章为脏 + 记录更新时间（编辑器修改回调） |
| `save_chapter_now(cid)` | 单章立即保存：重算字数 → 更新今日 stats → 加密写入 |
| `save_all()` | 批量 save_chapter_now + 写 novel.jsr |
| `settings_save()` | 加密写入 settings.jsr |
| `start_ai(action, messages)` | 启动流式 AI；入队用户消息 |
| `poll_ai(ctx)` | 每帧调用，收齐 Chunk → 刷新 UI，Done/Error 时 push 助手消息 |
| `editor_style()` | 合成 EditorStyle（字体/行距/字距/颜色…） |

### 7.3 shell.rs — eframe App 实现

> 路径：[src/shell.rs](file:///c:/Code/JinShu-rust/src/shell.rs)

`pub struct App { state: AppState, custom_titlebar: bool }`

**布局层次（从上到下，从左到右）**：

```
┌─────────────────────────── TitleBar (40px) ──────────────────────────┐
│  📖 锦书 · 《书名》                          [_][][x]  (Win自定义)   │
├──────┬──────────────┬──────────────────────────┬─────────────────────┤
│ 活动  │  侧边面板    │   中央区域(编辑器/详情页) │   AI 助手面板       │
│ 栏   │ Chapters     │   TabBar                 │   对话流 + 快捷动作 │
│ 48px │ Outline      │   ChapterHeader          │                     │
│      │ Characters   │   FindBar(可选)          │                     │
│      │ World        │   Editor                 │                     │
│      │ Timeline     │                          │                     │
│      │ Tasks/Search │                          │                     │
├──────┴──────────────┴──────────────────────────┴─────────────────────┤
│ StatusBar (26px)  🔒加密存储 | 字数 | 今日+N | 光标行:列 | 密钥指纹 │
└──────────────────────────────────────────────────────────────────────┘
```

**logic() 每帧调度**：
1. `poll_ai(ctx)` 收齐 AI 流式事件
2. 自动保存定时器（`autosave_secs` 到点 → `save_all()`）
3. toast 过期清理（4 秒）

**Command 枚举**（28 条命令）+ `execute_command()` 总线
- 文件：NewNovel / OpenLibrary / Save / Export / Import / CloseNovel
- 编辑：Find / Replace / GlobalSearch / NewChapter / NewVolume / CloseTab
- AI：AiContinue / AiOutline / AiPolish / AiExpand / AiSummary / AiChapterOutline / AiPlotIdeas / AiLogicCheck / AiConsistency / AiFeedback / AiCharacterCard / AiWorld / AiNaming / AiSynopsis
- 界面：ToggleSidebar / ToggleAI / ToggleTheme / FontUp / FontDown
- 其他：Stats / Settings / About

### 7.4 快捷键系统

| 快捷键 | 命令 |
|--------|------|
| Ctrl+N / Ctrl+O / Ctrl+S | 新建 / 书库 / 保存 |
| Ctrl+P | 命令面板 (Palette) |
| Ctrl+F / Ctrl+H / Ctrl+Shift+F | 查找 / 替换 / 全局搜索 |
| Ctrl+B / Ctrl+J | 切换侧边栏 / AI 面板 |
| Ctrl+= / Ctrl+- | 编辑区字号增 / 减 |
| Ctrl+W | 关闭当前标签 |

### 7.5 editor.rs — 自研编辑器组件

> 路径：[src/editor.rs](file:///c:/Code/JinShu-rust/src/editor.rs)

基于 egui `TextEdit` + 自定义 `LayoutJob`：
- 同步行号栏（逻辑行，自动换行时不错位）
- 可选 Markdown 语法高亮（`#` 标题、`*` 强调、` ``` ` 代码块）
- 行距 / 字距 / 字体 / 字号 独立控制
- 查找结果跳转：通过 `TextEdit::State` 设置 CCursor
- 编辑器输出 `EditorOutput { response, galley, cursor_range, changed }`

### 7.6 views/ — 侧栏与详情视图

| 模块 | 文件 | 功能 |
|------|------|------|
| 章节树 | views/chapters.rs | 卷/章嵌套列表，右键菜单（重命名/删除/上移/下移） |
| 大纲 | views/chapters.rs | 树结构，增删改 + 从 AI 回复解析缩进文本重建 |
| 人物 | views/world.rs | 列表 + 详情卡 + 关系网 Canvas（可拖拽布局） |
| 世界观 | views/world.rs | 地点层级树 + 详情编辑器 |
| 时间线 | views/world.rs | 事件排序卡片 + 关联章节 |
| 任务 | views/world.rs | 任务链切换 + 三列看板 Drag & Drop |
| 欢迎/书库/搜索/统计 | views/mod.rs | 欢迎页三卡片、书库列表、全局搜索、30天柱状图 |

---

## 8. Tauri 后端 (src-tauri/)

### 8.1 角色定位

Tauri 版后端是**无状态命令服务层**：前端通过 `invoke()` 调用 `#[tauri::command]` 函数，后端操作 Store 并返回结果。AI 流式和测试连接使用**事件推送**。

### 8.2 AppData 状态

```rust
pub struct AppData {
    store: Mutex<Store>,           // 顺序化并发访问
    cancel: Mutex<Option<Arc<AtomicBool>>>,  // AI 取消信号
}
```

### 8.3 命令清单 (invoke_handler)

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
| **导入导出** | | |
| `export_work` | `fmt, path, password?, novel, chapters` | `()` |
| `import_jsb` | `path, password` | `new_novel_id` |
| `open_dir` | `path` | `()` (explorer/xdg-open) |
| **AI** | | |
| `ai_start` | `cfg, messages` | `()` → 事件流 ai-chunk/done/error |
| `ai_cancel` | `()` | `()` |
| `ai_test` | `cfg` | `()` → 事件 ai-test-result |

### 8.4 AI 事件机制

```
前端 store.startAi()
  └─► invoke('ai_start')
        └─► Rust 端启动线程
              ├─ ureq 流式请求
              ├─ app.emit('ai-chunk', text)  ──► 前端 listen → aiStreamText += t
              ├─ app.emit('ai-done', ())     ──► 前端 push 完整消息
              └─ app.emit('ai-error', msg)   ──► 前端显示错误
```

前端在 `main.js` 注册事件监听器：
```js
listen("ai-chunk", ({payload}) => store.aiStreamText += payload);
listen("ai-done", ...);
listen("ai-error", ...);
listen("ai-test-result", ...);
```

---

## 9. Vue 3 Web 前端 (frontend/)

### 9.1 技术选型

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

### 9.2 响应式状态 (store.js)

与 egui 版 `AppState` 对应字段：
```js
store = reactive({
  ready, dataDir, keyFp, settings,
  library, novel, chapters,    // chapters = {cid: text}
  dirty, openTabs, activeTab,
  activity, sidebarOpen, aiPanelOpen, focusMode,
  paletteOpen, findOpen, findReplace,
  aiMsgs, aiStreaming, aiStreamText, aiAction, aiInput, useLore, summaries, aiTestResult,
  selChar, selLoc, selEvent, selOutline, selChain, showRelCanvas, canvasPos,
  dialog, toast, selectedText, cursorPos, wordCount, saveTimer, lastAutosave
})
```

### 9.3 业务动作 (store.js)

| 函数 | 作用 |
|------|------|
| `openNovel(id)` | invoke load_novel → 填充 chapters → 恢复上次章节 |
| `createNovel(form)` | invoke create_novel → 刷新书库 → 自动打开 |
| `closeNovel()` | saveAll + 回退书库 |
| `openTab / closeTab` | 标签页管理 + 保存设置 |
| `markDirty(cid)` | 置脏 + `scheduleSave()` |
| `scheduleSave()` | 2秒 debounce → saveAll |
| `saveChapterNow(cid)` | invoke save_chapter → 同步字数/stats/total |
| `saveAll()` | 遍历 dirty → 批量保存 + saveNovel |
| `saveSettings()` | invoke save_settings |
| `applyTheme()` | 设置 documentElement data-theme + CSS 变量（accent/ok/danger） |
| `startAi(action, messages)` | invoke ai_start → 等待事件 |
| `aiInsertToEditor(content)` | 触发 CustomEvent `jinshu:insert` → EditorView 在光标处插入 |
| `countWords(s)` | 中文字符 + 英文单词 双语种统计（JS 版镜像 util::count_words） |

### 9.4 组件装配 (App.vue)

```
<App>
  ├─ TitleBar            自定义标题栏 + 窗口控制 + 拖窗
  ├─ body (flex row)
  │   ├─ ActivityBar     10个活动图标按钮（非专注模式）
  │   ├─ SidePanel       侧栏内容（chapters/outline/characters/world/timeline/tasks/search）
  │   ├─ <main> central
  │   │   ├─ Library        activity=library
  │   │   ├─ StatsView      activity=stats
  │   │   ├─ SettingsView   activity=settings
  │   │   ├─ DetailViews    characters/world/timeline/tasks/outline 详情+画布
  │   │   ├─ EditorView     CodeMirror 6 编辑器（有 activeTab 时）
  │   │   └─ Empty 提示     无章节时
  │   └─ AIPanel        右侧 AI 助手面板（非专注模式）
  ├─ StatusBar
  ├─ Palette (条件)
  ├─ Modal (条件)
  ├─ ContextMenu
  └─ toast (fixed)
```

### 9.5 主题系统 (styles/theme.css)

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

### 9.6 CodeMirror 6 编辑器 (components/EditorView.vue)

核心特性：
- 初始化时安装 editorSettings 对应扩展（字体/行高/换行/行号/搜索/Markdown高亮）
- `view.dispatch({ changes: { insert } })` 响应 `jinshu:insert` 自定义事件
- `updateListener` 双向同步：Vue store.chapters[cid] ⇄ CM6 文档
- 选区变化 → `store.selectedText`
- 光标变化 → `store.cursorPos`

---

## 10. 导出 / 导入模块

> 路径：[src/export.rs](file:///c:/Code/JinShu-rust/src/export.rs) / [src-tauri/src/export.rs](file:///c:/Code/JinShu-rust/src-tauri/src/export.rs)

### 10.1 导出格式

#### txt 纯文本
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

#### md Markdown
```md
# 书名

> 作者：xxx

简介

---

## 第一卷

### 第一章 标题

正文……
```

#### jsb 加密备份
```jsonc
{
  "app": "jinshu-rust",
  "version": 1,
  "novel": { ...完整 Novel 对象... },
  "chapters": [{ "id", "title", "text" }, ...]
}
// → Zlib 压缩 → Scrypt(password, salt) 派生密钥 → AES-256-GCM
```

### 10.2 导入流程

```
import_jsb(path, password)
  ├─ 读取 .jsb 文件字节
  ├─ crypto::decrypt_jsb(password, bytes) → zlib → json 字符串
  ├─ 反序列化为 {novel, chapters}
  ├─ 重新分配 novel.meta.id（避免覆盖本地已有）
  ├─ Store::save_novel() → 逐章 Store::save_chapter()
  └─ 返回 new_novel_id
```

---

## 11. 工具函数与通用组件

> 路径：[src/util.rs](file:///c:/Code/JinShu-rust/src/util.rs) / [src/widgets.rs](file:///c:/Code/JinShu-rust/src/widgets.rs) / [src/theme.rs](file:///c:/Code/JinShu-rust/src/theme.rs)

### 11.1 util.rs 工具

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

### 11.2 theme.rs 主题

```rust
pub struct Palette {
    pub accent: Color32,        // 强调色（用户可选8种）
    pub bg_chrome/bg_panel/bg_panel_alt/bg_editor/bg_hover,
    pub text/text_secondary/text_disabled,
    pub ok/warn/danger,
}
impl Palette {
    pub fn dark(accent_rgb: [u8;3]) -> Self
    pub fn light(accent_rgb: [u8;3]) -> Self
}
pub fn install_fonts(ctx: &egui::Context, serif: bool)
  // 预加载 assets/fonts/NotoSansSC-VF.ttf / NotoSerifSC-VF.ttf
  // 解决 CJK 缺字；egui issue #5840 规避
```

### 11.3 widgets.rs 通用控件

```rust
pub fn activity_btn(...)        // 活动栏图标按钮（选中态）
pub fn icon_btn(...)            // 纯图标小按钮（标题栏/面板头）
pub fn secondary_btn(...)       // 次按钮样式（AI快捷操作）
pub fn h_sep / v_sep(...)       // 细分割线
pub fn empty_state(...)         // 空状态图标+标题+副标题
pub fn popup_frame(...)         // Palette/Modal 统一样式外框
```

---

## 12. 模块依赖关系图

### 12.1 egui 版依赖方向（自顶向下）

```
main.rs (入口)
  └─► shell.rs (eframe App)
        ├─► app.rs (AppState)
        │     ├─► model.rs      (Novel 数据模型 + 业务方法)
        │     ├─► store::mod.rs (加密 IO)
        │     │     └─► store::crypto.rs
        │     ├─► ai::client.rs (流式客户端)
        │     │     └─► ai::prompts.rs (提示词模板)
        │     ├─► editor.rs     (自定义编辑器)
        │     ├─► settings.rs   (设置页 UI)
        │     ├─► dialogs.rs    (对话框 UI)
        │     ├─► ai_panel.rs   (AI 面板 UI + 动作构造)
        │     ├─► views/*       (侧栏 + 详情视图)
        │     ├─► theme.rs      (Palette + 字体)
        │     ├─► widgets.rs    (通用控件)
        │     ├─► export.rs     (导出导入)
        │     └─► util.rs       (工具函数)
        └─► [直接引用 editor/theme/widgets/dialogs/views/ai_panel/settings]
```

**无循环依赖保证**：`model.rs` 只依赖 `util.rs`（ID + 时间）；`store/*` 依赖 `model.rs` + `crypto.rs`；`app.rs` 依赖两者之上。

### 12.2 Tauri 版依赖方向

```
frontend/*.vue + store.js
  └─► api.js (invoke 封装)
        └──► IPC ──► src-tauri/src/main.rs (#[tauri::command])
                              ├─► store.rs    (≈ src/store/mod.rs)
                              │     └─► crypto.rs
                              ├─► model.rs    (≈ src/model.rs)
                              ├─► ai_client.rs (≈ src/ai/client.rs)
                              │     └─► ai_prompts.rs
                              ├─► export.rs   (≈ src/export.rs)
                              └─► util.rs     (≈ src/util.rs)
```

### 12.3 外部 crates 依赖说明

| Crate | 用途 |
|-------|------|
| `eframe/egui 0.36` | 原生 GUI 框架 |
| `tauri 2` | WebView 桌面应用框架 |
| `serde + serde_json` | 所有数据模型序列化 |
| `aes-gcm 0.10` | AES-256-GCM AEAD 加解密（RustCrypto 项目） |
| `scrypt 0.11` | 密码派生密钥（.jsb 备份） |
| `sha2 0.10` | 密钥指纹 (SHA-256) |
| `rand 0.8` | 密钥/nonce/salt CSPRNG |
| `flate2 1` | .jsb 备份 Zlib 压缩 |
| `ureq 2` | 同步阻塞 HTTP 客户端（AI SSE） |
| `chrono 0.4` | 时间戳/日期格式化 |
| `uuid 1 (v4)` | 所有实体 ID 生成 |
| `rfd 0.15` | egui 版原生文件对话框 |
| `winresource 0.1` | Windows exe 图标+版本资源 |
| `vue 3.5` / `vite 6` | Tauri 前端 |
| `@codemirror/* 6.x` | Tauri 版编辑器 |
| `@tauri-apps/* 2.x` | Tauri 前端 API + CLI |

---

## 13. 核心数据流

### 13.1 编辑器输入 → 磁盘

```
用户输入字符
   │
   ▼
EditorView 或 egui editor
   │  out.changed == true
   ▼
AppState::mark_dirty(cid)
   │  dirty.insert(cid); meta_dirty = true; cmeta.updated_at = now
   ▼
shell.logic() / store.scheduleSave() 定时触发
   │
   ▼
AppState::save_chapter_now(cid) 或 saveAll()
   │  1. util::count_words(text) → 新字数
   │  2. today_stats += delta (新增部分)
   │  3. novel.meta.total_words = sum
   │  4. Store::save_chapter → crypto::encrypt_file(cid.jsr)
   │  5. Store::save_novel   → crypto::encrypt_file(novel.jsr)
   │  6. dirty.remove(cid)
   ▼
完成（加密文件落盘）
```

### 13.2 AI 请求 → 响应

```
用户点击"✨续写"按钮
   │
   ▼
ai_panel::build_continue_msgs(state, hint) → Vec<(role, content)>
   │  1. system_prompt (从 settings)
   │  2. 书名/体裁/简介
   │  3. 如果 settings.ai.inject_lore → lore_hits() 扫描末尾 3000 字
   │     → characters_to_text() + world_to_text() 注入
   │  4. 最近大纲（若有）
   │  5. 上一章末尾 + 当前章全文
   ▼
AppState::start_ai("续写", messages)
   │  1. push AiMsg::user 到 ai_msgs
   │  2. std::thread::spawn → ai::client::stream_chat
   ▼
Worker Thread: ureq POST SSE
   │  mpsc::channel → AiEvent::Chunk(t) 推送
   ▼
AppState::poll_ai(ctx) 每帧轮询
   │  ai_stream_text.push_str(&t) → 请求重绘
   │  流式展示中…
   │  ...
   │  AiEvent::Done → push AiMsg::assistant(ai_stream_text)
   ▼
用户可选：插入正文 / 复制 / 重新生成 / 应用为大纲 / 设为摘要
```

### 13.3 启动流程 (Tauri 版为例)

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
   ├─ 注册 17 条 commands
   └─ 启动 WebView → 加载 index.html
         │
         ▼
   Vue main.js
      ├─ api.initInfo() → {data_dir, key_fp, settings}
      ├─ store.settings = 结果
      ├─ applyTheme() → 设置 CSS 变量
      ├─ api.listNovels() → store.library
      ├─ 注册 4 个 AI 事件监听器
      ├─ 注册全局快捷键 (keydown → Command 分发)
      └─ createApp(App).mount()
            │
            ▼
         App.vue ready → 渲染
            └─ settings.last_novel_id → 自动 openNovel(id)
```

---

## 14. 构建与运行方式

### 14.1 从源码构建 Tauri 推荐版

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

### 14.2 构建 egui 原生版

```bash
# 在项目根目录
cargo build --release

# 产物：target/release/JinShu.exe (Windows 自动嵌入资源)
# 必须将 assets/ 目录放在 exe 同目录，否则 CJK 字体缺失
```

### 14.3 发布打包

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

#### Arch Linux 便携版
```bash
bash build_pkg_arch.sh
# 产物：dist/JinShu-rust-arch-x86_64.tar.gz → 解压直接 ./JinShu
```

### 14.4 开发调试

```bash
# Tauri 前端热更新
cd frontend && npm run dev    # Vite dev server @ http://localhost:5173
# 另开终端：cd src-tauri && cargo run (需指向 dev 前端，见 tauri.conf.json devUrl)

# egui 版直接
cargo run
```

### 14.5 运行测试

```bash
# egui 版 crate 根目录下：
cargo test
# 7 项测试：
#   - crypto_roundtrip：AES 加密往返 + 错误密钥必须失败 + AAD 绑定
#   - jsb_password_roundtrip：Scrypt + AES 密码备份往返
#   - store_encrypted_files：数据目录全盘加密验证（无明文文件）
#   - word_count_cjk：CJK/英文混排字数统计正确性
#   - novel_chapter_flow：卷/章 CRUD 业务
#   （另有 export 格式 / 大纲解析 测试，视源码版本）
```

---

## 15. 配置与环境变量

| 环境变量 | 作用 |
|----------|------|
| `JINSHU_DATA_DIR` | 强制指定数据目录，覆盖"exe同目录/data"默认行为。系统级安装时必须设置用户可写路径。 |
| `JINSHU_NATIVE_TITLEBAR` | egui 版 Windows 下设置为任意值即使用系统原生标题栏（禁用无边框自定义）。Linux 默认原生。 |

---

## 16. 测试覆盖

| 测试文件 | 测试名 | 验证点 |
|----------|--------|--------|
| `src/model.rs` | `word_count_cjk` | 汉字/标点/英文/空格混合统计边界 |
| `src/model.rs` | `novel_chapter_flow` | 新建卷→加章→查询→删除→计数同步 |
| `src/store/crypto.rs` | `crypto_roundtrip` | AES-GCM 加解对称 + 密钥错误/AAD 错误必失败 |
| `src/store/crypto.rs` | `jsb_password_roundtrip` | Scrypt 派生+Zlib+AES 密码备份往返 + 错误密码必失败 |
| `src/store/mod.rs` | `store_encrypted_files` | 1.无明文泄漏（字节扫描）；2.无 `.jsr` / `.jinshu_key` 外的扩展名；3.读回解密正确 |

**加密落盘测试的严格性**：遍历 `data_dir` 所有文件扩展名必须为 `jsr` 或名称为 `.jinshu_key`，保证不产生临时泄漏文件。

---

## 17. 快捷键与命令系统

### 17.1 Command 枚举总览

```rust
pub enum Command {
    // 文件类
    NewNovel, OpenLibrary, Save, Export, Import,
    // 编辑/导航类
    ToggleSidebar, ToggleAI, ToggleTheme, FontUp, FontDown,
    Find, Replace, GlobalSearch, NewChapter, NewVolume, CloseTab, CloseNovel,
    // AI 动作类（14 种）
    AiContinue, AiOutline, AiPolish, AiExpand, AiSummary, AiChapterOutline,
    AiPlotIdeas, AiLogicCheck, AiFeedback, AiConsistency, AiCharacterCard,
    AiWorld, AiNaming, AiSynopsis,
    // 视图类
    Stats, Settings, About,
}
```

### 17.2 命令面板 (Ctrl+P)

模糊匹配算法（egui 版）：
```rust
fn fuzzy_match(query: &str, label: &str) -> bool
// 按字符顺序包含（不需要连续），例如"xj"匹配"新建小说" x→i→a→n→**j**→i→a→n
```
Tauri 版：`components/Palette.vue` 中实现等价 `fuzzyMatch()` + 键盘方向键选择 + Enter 执行。

### 17.3 命令与快捷键映射表（最终版）

| 命令 | 快捷键 | 图标 / 入口 |
|------|--------|-------------|
| 新建小说 | Ctrl+N | 欢迎页卡片 / 命令面板 |
| 打开书库 | Ctrl+O | 活动栏 📚 |
| 保存全部 | Ctrl+S | 标题栏 💾 / 章节头按钮 |
| 导出作品 | — | 命令面板 / 对话框 |
| 导入备份 | — | 欢迎页卡片 / 命令面板 |
| 查找 | Ctrl+F | 章节内查找条 |
| 查找替换 | Ctrl+H | 查找条展开替换行 |
| 全局搜索 | Ctrl+Shift+F | 活动栏 🔍 / 侧栏 |
| 命令面板 | Ctrl+P | 屏幕中央悬浮搜索 |
| 切换侧边栏 | Ctrl+B | — |
| 切换 AI 面板 | Ctrl+J | — |
| 切换深/浅色 | — | 设置页或命令面板 |
| 增大字号 | Ctrl+= | — |
| 减小字号 | Ctrl+- | — |
| 关闭标签页 | Ctrl+W | 标签右键菜单 |
| 新建章节 | — | 章节面板 ➕ |
| 新建分卷 | — | 大纲面板 ➕ |
| 写作统计 | — | 活动栏 📊 |
| 设置 | — | 活动栏 ⚙️ |
| AI 续写 | — | 章节头 ✨续写 / 命令面板 |
| AI 其他 13 项 | — | AI 面板顶栏下拉菜单 / 命令面板 |

---

## 附录：安全边界说明

> 摘自 README 并补充实现细节

1. **密钥文件**：`data/.jinshu_key` 32字节纯随机，Unix 权限 0o600。删除此文件后旧数据**无法解密**，应配合 `.jsb` 密码备份作为灾备。
2. **AAD 绑定**：加密时使用文件名作为附加认证数据，防止攻击者将"章A密文"重命名为"章B密文"后被解密为章B的正文冒充。
3. **原子写入**：所有文件先写 `*.tmp` 再 rename，保证断电崩溃不损坏原文件（只可能丢失最近一次）。
4. **不污染系统**：不写 `%APPDATA%`、`%TEMP%`、`~/.local`、`/tmp`（已验证：所有持久文件扩展名均为 `.jsr` 或名为 `.jinshu_key`）。
5. **威胁模型**：
   - 可防：随手读盘、同事借用、备份盘丢失、云盘同步时的明文泄漏。
   - 不可防：同权限进程注入读内存、内存dump、物理机调试类攻击（密钥与数据同机）。
   - 高强度方案：定期导出 `.jsb` 密码备份（密钥由 Scrypt 从密码派生，密码不落盘）。

---

*本 Code Wiki 对应代码版本：仓库 HEAD（2026-09-02 分析快照）*
