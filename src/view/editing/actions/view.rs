// mochou-p/text-editor/src/view/editing/actions/view.rs

use crate::Editor;


impl super::super::Editing {
    pub fn scroll(&mut self, editor: &mut Editor, direction: isize) {
        match direction {
            -1 => self.scroll_up(),
            1  => self.scroll_down(editor),
            _  => ()
        }
    }

    fn scroll_down(&mut self, editor: &mut Editor) {
        let Some(file) = self.file.as_ref() else { return; };

        if self.scroll.y != (editor.files[file].lines.len() - 1) as isize {
            self.scroll.y += 1;
        }
    }

    fn scroll_up(&mut self) {
        if self.file.is_none() { return; };

        if self.scroll.y != 0 {
            self.scroll.y -= 1;
        }
    }
}
