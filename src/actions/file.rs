// text-editor/src/actions/file.rs

use crate::Editor;


pub fn save(editor: &mut Editor) {
    if let Some(file) = &editor.file {
        if editor.cursor.y != 0 && editor.lines.last().unwrap().is_empty() {
            editor.lines.push(String::new());
        }

        std::fs::write(file, editor.lines.join("\n")).unwrap();
    }
}

