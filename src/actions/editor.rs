// mochou-p/text-editor/src/actions/editor.rs

use crate::Editor;


pub fn exit(editor: &mut Editor) {
    editor.exit = true;
}

