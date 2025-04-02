// text-editor/src/main.rs

#![expect(clippy::multiple_crate_versions, reason = "crossterm")]

mod editor;

use std::error::Error;

use editor::TextEditor;


fn main() -> Result<(), Box<dyn Error>> {
    let mut te = TextEditor::new()?;
    te.run()
}

/* FIXME
do not crash on max width/height, impl overflow & scroll
sometimes alternate screen somehow doesnt clear terminal
*/

