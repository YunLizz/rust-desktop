#!/usr/bin/env bash
# 锦书 · 小说编辑器（Tauri v2）—— Windows 便携打包脚本
# 用法: bash build_pkg_win.sh   （在项目根目录执行）
set -e
cd "$(dirname "$0")"
PKG="dist/JinShu-win64"

echo "==> 前端构建"
(cd frontend && npm run build)

echo "==> 后端 Release 构建"
(cd src-tauri && cargo build --release)

echo "==> 组装 $PKG"
rm -rf "$PKG"
mkdir -p "$PKG"
cp src-tauri/target/release/jinshu.exe "$PKG/JinShu.exe"
cp README.md "$PKG/README.md"
[ -f "dist/JinShu-win64/启动说明.txt" ] && cp "dist/JinShu-win64/启动说明.txt" "$PKG/" || true

cat > "$PKG/启动说明.txt" <<'EOF'
锦书 · 小说编辑器（Windows 便携版 · Tauri）
============================================

使用：双击 JinShu.exe 即可运行，无需安装（Win10/11 自带 WebView2）。

数据与安全：
· 所有作品、章节、设置均以 AES-256-GCM 加密文件存储在本目录 data/ 文件夹，
  不会写入 %APPDATA%、%TEMP% 等系统目录。
· 解密密钥 = data/.jinshu_key（随机 32 字节），请勿删除；移动整个文件夹即随身携带数据。
· 磁盘上不存在任何明文文件（已自动化测试验证）。

AI 功能：
· 设置 → AI 服务：填写任意 OpenAI 兼容 / Anthropic 兼容服务的 API Key。
· API Key 加密存储，仅在你调用时发送给你配置的服务商。

快捷键：
Ctrl+N 新建小说   Ctrl+O 书库   Ctrl+S 保存   Ctrl+P 命令面板
Ctrl+F 查找       Ctrl+B 侧栏   Ctrl+J AI 面板  Ctrl+=/Ctrl+- 字号
EOF

echo "==> 压缩 zip"
cd dist
rm -f JinShu-rust-win64-tauri.zip
powershell -Command "Compress-Archive -Path JinShu-win64 -DestinationPath JinShu-rust-win64-tauri.zip -Force" 2>/dev/null \
  || python -c "import shutil; shutil.make_archive('JinShu-rust-win64-tauri', 'zip', 'JinShu-win64')"
echo "==> 完成: dist/JinShu-rust-win64-tauri.zip"
ls -la dist/*.zip
