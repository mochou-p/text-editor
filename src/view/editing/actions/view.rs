// mochou-p/text-editor/src/view/editing/actions/view.rs

use crate::view::View;


impl super::super::Editing {
    pub fn scroll_dir(&mut self, direction: isize) {
        match direction {
            -1 => self.scroll_up(),
            1  => self.scroll_down(),
            _  => ()
        }
    }

    fn scroll_down(&mut self) {
        let Some(file) = self.file.as_ref() else { return; };

        if self.scroll().y != (self.files[file].lines.len() - 1) as isize {
            self.scroll_mut().y += 1;
        }
    }

    fn scroll_up(&mut self) {
        if self.file.is_none() { return; };

        if self.scroll().y != 0 {
            self.scroll_mut().y -= 1;
        }
    }
}
