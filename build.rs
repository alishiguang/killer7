use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=assets/fonts");
    println!("cargo:rustc-check-cfg=cfg(has_display_font)");
    println!("cargo:rustc-check-cfg=cfg(has_mono_font)");

    // 字体是可选增强：缺失时回退到系统字体，不影响编译。
    if Path::new("assets/fonts/Anton-Regular.ttf").exists() {
        println!("cargo:rustc-cfg=has_display_font");
    }
    if Path::new("assets/fonts/GeistMono-Variable.ttf").exists() {
        println!("cargo:rustc-cfg=has_mono_font");
    }
}
