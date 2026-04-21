// mochou-p/text-editor/src/view/editing/actions/file.rs

use std::io::Write as _;

use crate::Editor;


impl super::super::Editing {
    pub fn save(&mut self, editor: &mut Editor) {
        let Some(file) = self.file.as_ref() else { return; };

        if editor.files[file].clean {
            return;
        }

        // NOTE: 7 -> '\a' -> BEL
        write!(editor.stdout, "{}", 7 as char).unwrap();
        editor.stdout.flush().unwrap();

        let writee = if
            editor.files[file].lines.len() == 1
            &&
            editor.files[file].lines[0].is_empty()
        {
            String::new()
        } else {
            editor.files[file].lines.join("\n")
        };

        std::fs::write(file, writee).unwrap();
        editor.files.get_mut(file).unwrap().clean = true;
    }
}
