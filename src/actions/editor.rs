// text-editor/src/actions/editor.rs

use crate::Editor;


pub const fn exit(editor: &mut Editor) {
    editor.exit = true;
}

