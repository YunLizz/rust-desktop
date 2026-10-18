# 锦书 JinShu — 构建计划与路线图（ROADMAP）

> **用途**：汇总项目的构建/交付计划、完成状态与后续路线，供换环境、换 Agent 交接使用。
> **最后更新**：2026-10-18
> **当前 HEAD**：`63f89bb`
> **配套文档**：`HANDOFF_SUMMARY.md`（一页摘要）、`CONTEXT_HANDOFF.md`（详细上下文）、`README.md`（项目介绍）

---

## 0. 状态速览

| 维度 | 进度 |
|---|---|
| 版本 | `0.1.0`（Cargo / tauri.conf / package.json 三处一致） |
| 构建与交付线 | **6 / 10 完成**（A1–A6 完成，A7 待验证，A8–A10 未做） |
| 功能演化阶段 | **8 / 8 阶段完成** |
| 已实现功能 | 14 项（见第 2 节） |
| 待办 | 8 项（见第 3 节） |
| 发布状态 | 无 git tag、无 GitHub Release、无 CI |

> 完成度是「计划项状态」的计数，不是工作量百分比。

---

## 1. 构建与交付线（6 / 10）

| # | 计划项 | 状态 | 实测证据 / 说明 |
|---|---|---|---|
| A1 | 前端生产构建 | ✅ | `vite build` 2.08s（2026-10-18） |
| A2 | Rust release 构建 | ✅ | 53.6s，产物 `src-tauri/target/release/jinshu.exe` |
| A3 | 单元测试 | ✅ 6/6 | `cargo test`（2026-10-18；文档此前误写 7 个，已修正） |
| A4 | Windows 便携 zip | ✅ 已存在 | `dist/JinShu-rust-win64-tauri.zip`（4.16MB，2026-09-12 构建；Tauri 应用源码此后无功能变更——`6586428` 仅删 egui 遗留文件，功能等价，但包内 README 较旧） |
| A5 | Windows NSIS 安装包 | ✅ | 2026-10-18 构建+实测，3,098KB；产物在 `target/`（不入库） |
| A6 | 安装/运行/加密/卸载端到端 | ✅ | 装到 `$EXEDIR`、不写用户目录、磁盘零明文、卸载保留 `data/`（详见 `CONTEXT_HANDOFF.md` 第十一节） |
| A7 | Arch PKGBUILD | ⚠️ 已写未验证 | `url` 仍是 `github.com/example/...` 占位；`maintainer` 为 example 邮箱；`source` 无真实下载地址、`sha256sums` 为 SKIP |
| A8 | CI（自动测试/构建） | ❌ | 无 `.github/` |
| A9 | 版本 tag / GitHub Release | ❌ | `git tag` 为空，无 Release |
| A10 | 代码签名 / 自动更新 | ❌ | 未做（离线工具，非必需，优先级最低） |

---

## 2. 功能演化线（8 / 8 阶段完成）

| 阶段 | 内容 | 状态 |
|---|---|---|
| 1 | Rust + egui 从零实现 | ✅ |
| 2 | 发现 egui 打包后中文字体加载失败 | ✅（问题已定位） |
| 3 | 重构为 Tauri（包体 25MB → 4.2MB） | ✅ |
| 4 | 融合 Python 版特性（排版/任务拖拽/章节骨架） | ✅ `b1873d4` |
| 5 | 移除 `.jsb` 备份、前端屏蔽 AI（后端保留） | ✅ `7d8e980` |
| 6 | 前端按「人眼专注区」重构四区布局 | ✅ `ab73b5d` |
| 7 | 任务线语义修正 + 剪辑式横向时间轴 | ✅ `37a403b` |
| 8 | 删除 egui 遗留，项目转 Tauri 单引擎 | ✅ `6586428` |

**已实现功能（14 项）**：书库 / 章节树（卷章） / 大纲树（含从章节生成骨架） / 人物树 + 关系网 / 世界树 + 地图画布 / 任务线 / 剪辑式横向时间轴 / CodeMirror 编辑器 / 本地起名机 / 导出 txt+md / 深浅主题 + 8 强调色 / 命令面板 / 自动保存 / 加密存储。

**屏蔽/移除**：AI 功能（前端移除、后端 `ai_client.rs`/`ai_prompts.rs`/命令/`AiSettings`/`ureq` 全部保留，用户明确后续恢复）；`.jsb` 备份已移除；egui 版已删除。

---

## 3. 待办清单（按优先级）

| 优先级 | 事项 | 来源 | 说明 |
|---|---|---|---|
| P0 | 卸载「删除应用数据」勾选无效 | 2026-10-18 实测 | 模板删的是 `%APPDATA%/%LOCALAPPDATA%\{bundleId}`，真实数据在 `$INSTDIR\data` |
| P0 | 发布物补齐（刷新便携 zip、tag、GitHub Release、sha256） | 构建线 A4/A9 | 当前只能从 git 拿源码自行构建 |
| P0 | 文档同步（测试数、下载/安装方式、Release 链接） | 核对发现 | 测试数已在本次修正 |
| P1 | Arch 真机验证 | 交接待办 | 先修 PKGBUILD 占位 URL / source / 校验和 |
| P2 | AI 功能恢复 | 交接文档 | 前端组件从 `b1873d4` 取回，需先定交互方案 |
| P2 | 时间轴支持手动指定轨道 | 交接文档 | 按章节分轨时未关联事件都挤在“未关联章节”轨 |
| P2 | 起名机字库扩充 | 交接文档 | 现约 300 字 |
| 可选 | GitHub Actions CI；签名/自动更新 | 工程化 | 优先级最低 |

---

## 4. 后续计划

### 阶段 1 · 发布就绪（P0，约半天~1 天）

1. 修复卸载「删除应用数据」勾选：改为删除 `$INSTDIR\data`（更新/卸载时按勾选决定是否保留作品）
2. 重新构建 NSIS 安装包 → 静默安装/卸载复测（含数据保留用例）
3. 用当前源码重建便携 zip（更新包内 `启动说明.txt` 与 README）
4. 打 `v0.1.0` tag；创建 GitHub Release，附 zip + installer + sha256
5. 同步文档：下载方式、Release 链接

**验收**：全新目录安装 → 新建作品并写入 → 卸载（不勾选删数据）→ 重装后作品仍在；Release 页面可直接下载。

### 阶段 2 · 跨平台（P1，需 Linux 环境，约 1 天）

1. 修 PKGBUILD：真实仓库 URL、`source` 指向 GitHub tag tarball、补 `sha256sums`、核对依赖
2. Arch 真机 `makepkg -si` → 启动 → 中文显示 → 加密落盘验证
3. 明确 `/opt` 不可写时的 `JINSHU_DATA_DIR` 用法

**验收**：Arch 上安装后可正常写作，数据落在安装目录（或指定的 `JINSHU_DATA_DIR`）。

### 阶段 3 · 功能（P2，需用户确认交互方案，约 1~3 天）

1. AI 恢复：先出交互方案（入口位置、模型配置、离线/在线切换、错误提示），用户确认后接回前端组件与 `ai-*` 事件
2. 时间轴手动指定轨道
3. 起名机字库扩充

### 阶段 4 · 工程化（可选）

1. GitHub Actions：`cargo test` + 前端构建 + `tauri build`，产物上传 artifact
2. 代码签名 / 自动更新（离线工具暂不需要）

**依赖关系**：阶段 1 可独立完成；阶段 2 需要 Linux 机器；阶段 3 依赖用户对 AI 交互的决策；阶段 4 可与其它阶段并行。

---

## 5. 交接关键事实（避免踩坑）

- **测试实际是 6 个**（`crypto_roundtrip`、`store_encrypted_files`、`export_formats`、`word_count_cjk`、`novel_chapter_flow`、`outline_parse`）；交接文档曾误写 7 个，已修正。
- **安装包 exe 不入库**（在 `target/`，已 gitignore）：新机器需重建，命令见 `CONTEXT_HANDOFF.md` 第十一节；首次构建会自动下载 NSIS 工具链。
- **便携 zip 在仓库内**（`dist/`），但停在 2026-09-12；Tauri 应用源码此后无功能变更（`6586428` 仅删除 egui 遗留文件），功能与当前一致。
- **PKGBUILD 有占位内容**，直接 `makepkg` 前必须先改。
- **无 tag / Release / CI**；发布流程需从阶段 1 建立。
- **凭据永不落盘**；Git 凭据在系统凭据管理器。
- **构建前先关掉正在运行的 JinShu**（安装/卸载时模板会检测运行中实例）。

---

## 6. v0.1.0 发布检查清单

- [ ] 卸载「删除应用数据」勾选修复并复测
- [ ] NSIS 安装包重新构建 + 端到端复测
- [ ] 便携 zip 用当前源码重建
- [ ] `git tag v0.1.0` 并推送
- [ ] GitHub Release（zip + installer + sha256）
- [ ] README / HANDOFF / CONTEXT 同步下载与安装方式
- [ ] （可选）GitHub Actions CI 加入
