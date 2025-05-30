// text-editor/src/main.rs

#[cfg(not(target_os = "linux"))]
compile_error!("this project only supports Linux");

mod ansi;
mod editor;

use {
    ansi::{color::{RESET, FOREGROUND_RED}, CSI},
    editor::Editor
};


fn main() {
    Editor::default().and_or(
        Editor::run,
        |error| eprintln!("{CSI}{FOREGROUND_RED}error{CSI}{RESET}: {error}")
    );
}

