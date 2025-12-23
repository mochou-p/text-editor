// text-editor/src/actions/file.rs

use std::io::Write as _;

use crate::Editor;


pub fn save(editor: &mut Editor) {
    if editor.clean {
        return;
    }

    if let Some(file) = &editor.file {
        // NOTE: BEL (\a)
        write!(editor.stdout, "{}", 7 as char).unwrap();
        editor.stdout.flush().unwrap();

        if editor.cursor.y != 0 && editor.lines.last().unwrap().is_empty() {
            editor.lines.push(String::new());
        }

        std::fs::write(file, editor.lines.join("\n")).unwrap();

        editor.clean = true;
    }
}

