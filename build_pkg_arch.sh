#!/usr/bin/env bash
# 锦书 · 小说编辑器 —— Arch Linux 安装脚本
# 用法: bash build_pkg_arch.sh   （在项目根目录执行）
# 产物: dist/JinShu-rust-arch-x86_64.tar.gz（可解压到任意目录运行）
set -e
cd "$(dirname "$0")"

echo "==> 构建 Release（需要 Rust 工具链）"
cargo build --release

PKG="dist/JinShu-rust-arch-x86_64"
echo "==> 组装目录 $PKG"
rm -rf "$PKG"
mkdir -p "$PKG/assets/fonts"
cp target/release/jinshu-rust "$PKG/JinShu"
cp assets/fonts/*.ttf "$PKG/assets/fonts/" 2>/dev/null || true
cp README.md "$PKG/README.md" 2>/dev/null || true

cat > "$PKG/启动说明.txt" <<'EOF'
锦书 · 小说编辑器（Arch Linux 便携版）
========================================

运行：
  ./JinShu
  或安装到 ~/Apps 后创建桌面快捷方式：
    [Desktop Entry]
    Name=锦书
    Exec=/home/你/Apps/JinShu-rust-arch-x86_64/JinShu
    Terminal=false
    Type=Application
    Categories=Office;

数据与安全：
  · 数据目录默认 = 本目录/data（安装目录内，加密存储，不入 ~/.local 等系统目录）。
  · 若本目录无写权限，设置环境变量：JINSHU_DATA_DIR=/你的路径/data
  · 解密密钥 = data/.jinshu_key（随机 32 字节，请勿删除）

依赖（Arch 默认已满足大部分）：
  图形: libxkbcommon / wayland / x11
  如需原生文件对话框: gtk3
  中文字体: 已内置 Noto Sans/Serif SC（assets/fonts），无需额外安装

打包为 Arch 官方格式：
  使用 packaging/arch/PKGBUILD（makepkg -si）
EOF

echo "==> 压缩"
cd dist
rm -f JinShu-rust-arch-x86_64.tar.gz
tar -czf JinShu-rust-arch-x86_64.tar.gz JinShu-rust-arch-x86_64
echo "==> 完成: dist/JinShu-rust-arch-x86_64.tar.gz"
ls -la dist/
