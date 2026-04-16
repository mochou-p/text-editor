// mochou-p/text-editor/src/actions/view.rs

use crate::Editor;


pub fn scroll_down(editor: &mut Editor) {
    if editor.view.scroll.y != (editor.files[&editor.view.file].lines.len() - 1) as isize {
        editor.view.scroll.y += 1;
    }
}

pub fn scroll_up(editor: &mut Editor) {
    if editor.view.scroll.y != 0 {
        editor.view.scroll.y -= 1;
    }
}
