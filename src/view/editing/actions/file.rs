// mochou-p/text-editor/src/view/editing/actions/file.rs

use std::io::Write as _;

use crate::Editor;


impl super::super::Editing {
    pub fn save(&mut self, editor: &mut Editor) {
        let Some(file) = self.file.as_ref() else { return; };

        if self.files[file].clean {
            return;
        }

        // NOTE: 7 -> '\a' -> BEL
        write!(editor.stdout, "{}", 7 as char).unwrap();

        let writee = if
            self.files[file].lines.len() == 1
            &&
            self.files[file].lines[0].is_empty()
        {
            String::new()
        } else {
            self.files[file].lines.join("\n")
        };

        std::fs::write(file, writee).unwrap();
        self.files.get_mut(file).unwrap().clean = true;
    }
}
