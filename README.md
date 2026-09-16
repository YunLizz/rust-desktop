# 📖 锦书 · 小说编辑器（JinShu-rust）

为中文小说创作而生的本地编辑器。**Tauri v2（Rust 后端 + Vue 3 Web 前端）**构建，现代化 IDE 风格界面，章节树 / 大纲 / 人物关系网 / 世界观 / 时间线 / 任务看板 / 写作统计开箱即用，AI 创作助手可接入你自己的大模型 API Key，**全部数据以 AES-256-GCM 加密文件缓存于软件安装目录**，不写入任何系统目录。

![界面](docs/screenshot.png)

---

## 🖥️ 界面设计：按人眼专注区做空间分区

界面按「人眼在屏幕上的专注度分布」划分为四个区域（2K 27 寸显示器实测调优）：

| 区域 | 内容 | 设计理由 |
| --- | --- | --- |
| **中央** | 正文编辑区（约 600-720px，约 34 字/行） | 人眼专注区（屏幕中央约 10 寸），只放正文，宽度自适应：屏宽 45% 且不超过 720px，可调 |
| **左侧栏** | 4 棵树 Tab 切换：**章节 / 大纲 / 人物 / 世界观** | 所有需要层级展开/折叠的内容（树形结构） |
| **顶部 Ribbon** | 分组选项卡（写作/设定/工具/视图）+ **可折叠横向时间轴** | 功能类操作 + 天然横向展开的时间轴、任务脉络 |
| **右侧栏** | **创意工具格**：起名机 / 地图 / 关系网 / 时间轴 / 任务 | 随用随开的创意工具，写作时不离开正文 |

**关键改进**：
- **三栏宽度全部可拖拽**（拖分隔条调整、双击恢复默认、宽度记忆到设置）
- **写作时无需翻页**：左侧章节树始终可见，右侧工具格点开即用，不再在 10 个页面间反复跳转
- **顶栏横向时间轴**：一键展开，把剧情事件按时间平铺在一条横线上，当前章节对应节点高亮，点击跳转
- **正文不铺满屏幕**：中文单行 30-40 字最佳，过宽会导致眼睛长距离横扫

## 🔒 数据安全模型

- **加密缓存**：所有作品、章节、设置（含 AI API Key）均以 **AES-256-GCM** 加密文件存储于**安装目录** `data/` 下，磁盘上不存在任何明文文件（已由自动化测试验证）。
- **融合说明**：本版本已融合原 Python 版（JInshu-py）的全部功能与 Word 式中文排版特性（首行缩进/两端对齐/楷体仿宋字体/任务拖拽/章节骨架/md 导出含大纲）。
- **密钥管理**：首次运行在 `data/.jinshu_key` 生成随机 32 字节密钥；密钥指纹显示于状态栏与设置页。
- **不污染系统**：不向 `%APPDATA%`、`~/.local`、`%TEMP%` 等系统目录写入任何持久文件。
- **安全边界**：本方案防御"随手读盘"类威胁（其他程序直接读文件只能看到密文）。由于密钥与数据同机，无法防御针对性的同权限攻击；如需更强保护，请使用导出 `.jsb` 加密备份（密码不落盘）。
- **数据目录位置**：可执行文件同目录 `data/`（可通过环境变量 `JINSHU_DATA_DIR` 覆盖）。

## 🤖 AI 服务接入（自带 Key，不经过任何中转）

`设置 → AI 服务`：

| 项目 | 说明 |
| --- | --- |
| 协议 | OpenAI 兼容（绝大多数服务）或 Anthropic |
| 服务商预设 | DeepSeek、Moonshot、通义千问、智谱 GLM、Ollama 本地、自定义 |
| Base URL | 中转/代理可自定义 |
| 模型 | 任意模型名，如 `deepseek-chat`、`qwen-plus`、`claude-sonnet-4-5` |
| API Key | 掩码显示，加密落盘，仅在你调用时发送给你配置的服务商 |

**Lorebook 注入**：开启"自动注入设定"后，AI 请求会自动携带当前正文中出现的人物/地点设定卡，保证角色言行一致。

## 🚀 快速开始

### Windows 11

1. 下载 `JinShu-rust-win64-tauri.zip`（约 4MB），解压到任意目录（如 `D:\Apps\JinShu`）
2. 双击 `JinShu.exe`（Win10/11 自带 WebView2，无需安装任何运行时）
3. 数据保存在 exe 同目录 `data/`，整个文件夹可随身携带

### Arch Linux

**方式一：AUR 风格打包（推荐）**

```bash
cd packaging/arch
makepkg -si        # 安装到系统
```

> 注意：系统级安装时 `/opt/jinshu-rust` 默认不可写，请在启动前设置 `JINSHU_DATA_DIR=~/jinshu-data`（或 `sudo chown -R $USER /opt/jinshu-rust`，数据即可直接落在安装目录）。

**方式二：便携版**

```bash
bash build_pkg_arch.sh   # 在项目根目录（需要 Rust 工具链）
# 产物: dist/JinShu-rust-arch-x86_64.tar.gz —— 解压到 ~/Apps 直接运行
./JinShu
```

依赖：`webkit2gtk-4.1 gtk3 noto-fonts-cjk`（PKGBUILD 已声明）；中文字体来自系统 noto-fonts-cjk。

### 从源码构建

```bash
# 1. 前端
cd frontend && npm install && npm run build && cd ..
# 2. 后端（自动嵌入前端产物）
cd src-tauri && cargo build --release
./src-tauri/target/release/jinshu   # Windows: jinshu.exe
```

需要 Rust 1.85+ 与 Node 20+。

## ⌨️ 快捷键

| 快捷键 | 功能 |
| --- | --- |
| Ctrl+N / Ctrl+O / Ctrl+S | 新建小说 / 打开书库 / 保存 |
| Ctrl+P | 命令面板 |
| Ctrl+F / Ctrl+H / Ctrl+Shift+F | 查找 / 替换 / 全局搜索 |
| Ctrl+B / Ctrl+J | 切换侧边栏 / AI 助手面板 |
| Ctrl+= / Ctrl+- | 编辑区字号增减 |
| Ctrl+W | 关闭当前标签页 |

## 🧪 测试

```bash
cargo test    # 加密往返 / 密码备份 / 字数统计 / 导出格式 / 大纲解析 / 加密落盘 7 项
```

## 🏗 技术栈

Rust + Tauri 2 · Vue 3 · CodeMirror 6（编辑器：行号/高亮/查找替换/撤销历史）· AES-256-GCM（RustCrypto）· Scrypt（.jsb 备份）· ureq（AI 流式客户端）

## 📄 许可

MIT —— 完全开源，可自由商用。
