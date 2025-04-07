// text-editor/src/main.rs

#![expect(clippy::multiple_crate_versions, reason = "inside crossterm")]

mod config;
mod editor;
mod utils;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    editor::TextEditor::new()?
        .run()
}

