fn main() {
    // build.rs 运行在编译主机上，所以不能用 cfg!(windows) 做条件（那是 HOST 平台），
    // 必须通过 CARGO_CFG_TARGET_OS 环境变量判断 TARGET 平台。
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "windows" {
        let mut res = winresource::WindowsResource::new();
        // 资源文件相对项目根目录（即 Cargo.toml 所在目录）。
        // 若 icons/app.ico 存在，则嵌入；否则跳过图标嵌入，只注入版本信息。
        let icon_path = "assets/icons/app.ico";
        if std::path::Path::new(icon_path).exists() {
            res.set_icon(icon_path);
        }
        // 语言：中文简体 (LCID = 0x0804)
        res.set_language(0x0804);
        // ProductVersion / FileVersion / 描述 / 版权 等优先读取 Cargo.toml 的
        // [package.metadata.winresource]；这里再兜底一次。
        res.set("CompanyName", "JinShu Team");
        res.set("ProductName", "锦书 JinShu");
        res.set("FileDescription", "锦书 · 现代化小说编辑器");
        res.set("LegalCopyright", "Copyright © 2026 JinShu Team");

        if let Err(e) = res.compile() {
            // 资源编译失败不应阻塞主构建：打印到 stderr 让用户知道，然后继续
            eprintln!("cargo:warning=winresource compile failed, skipped: {e}");
        }
    }
    // 告诉 cargo：当 assets/icons/ 或 build.rs 自己变化时重跑 build 脚本
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/icons");
}
