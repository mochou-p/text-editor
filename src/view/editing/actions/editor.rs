// mochou-p/text-editor/src/view/editing/actions/editor.rs

impl super::super::Editing {
    pub fn exit(&mut self, editor: &mut crate::Editor) {
        editor.exit = true;
    }
}
