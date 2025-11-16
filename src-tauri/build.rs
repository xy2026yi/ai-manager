fn main() {
    let mut context = tauri_build::BuildContext::new_default();

    // 启用资源优化
    println!("cargo:rerun-if-changed=icons");
    println!("cargo:rerun-if-changed=src-tauri/tauri.conf.json");

    // 构建时配置
    tauri_build::build()
}
