// text-editor/src/main.rs

#[cfg(not(target_os = "linux"))]
compile_error!("this project currently only supports Linux");

mod ansi;
mod editor;


fn main() -> std::io::Result<()> {
    editor::Editor::run()
}

