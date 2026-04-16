// mochou-p/text-editor/src/actions/file.rs

use std::io::Write as _;

use crate::Editor;


pub fn save(editor: &mut Editor) {
    if editor.files[&editor.view.file].clean {
        return;
    }

    // NOTE: 7 -> '\a' -> BEL
    write!(editor.stdout, "{}", 7 as char).unwrap();
    editor.stdout.flush().unwrap();

    let writee = if
        editor.files[&editor.view.file].lines.len() == 1
        &&
        editor.files[&editor.view.file].lines[0].is_empty()
    {
        String::new()
    } else {
        editor.files[&editor.view.file].lines.join("\n")
    };

    std::fs::write(&editor.view.file, writee).unwrap();
    editor.files.get_mut(&editor.view.file).unwrap().clean = true;
}
