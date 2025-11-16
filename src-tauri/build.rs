fn main() {
    // 启用资源优化
    println!("cargo:rerun-if-changed=icons");
    println!("cargo:rerun-if-changed=src-tauri/tauri.conf.json");

    // 构建 Tauri 应用
    // 使用 Tauri 1.5 兼容的 API
    tauri_build::build()
}
