// 发布构建下隐藏终端窗口（macOS 无此效果，但保持 Windows/Linux 兼容写法一致）。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    chuaker7_lib::run();
}
