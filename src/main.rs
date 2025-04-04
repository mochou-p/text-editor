// text-editor/src/main.rs

#![expect(clippy::multiple_crate_versions, reason = "inside crossterm")]

mod config;
mod editor;

use std::error::Error;

use editor::TextEditor;


fn main() -> Result<(), Box<dyn Error>> {
    TextEditor::new()?
        .run()
}

