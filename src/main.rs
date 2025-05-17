// text-editor/src/main.rs

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

