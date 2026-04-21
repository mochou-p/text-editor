// mochou-p/text-editor/src/view/files.rs

use std::io::Write as _;
use betterm::cursor;
use super::{View, ViewData};
use crate::Editor;


pub struct Files {
    view_data: ViewData
}

impl Files {
    pub fn new(editor: &mut Editor) -> Self {
        Self { view_data: ViewData::above(editor, "editing", 1) }
    }
}

impl View for Files {
    fn name     (         ) ->      String   { String::from("files") }
    fn view_data(&mut self) -> &mut ViewData { &mut self.view_data   }

    fn reprint(&mut self, editor: &mut Editor) {
        let text = format!(" placeholder ");

        write!(
            editor.stdout,
            "{}{}{}{}{}{}",
            cursor::MoveToColumnAndRow(
                (self.position().x + 1) as u16,
                (self.position().y + 1) as u16
            ),
            editor.theme.backgrounds.primary.normal,
            editor.theme.foreground.active,
            text,
            editor.theme.backgrounds.primary.disabled,
            " ".repeat(self.size().x as usize - text.len())
        ).unwrap();
    }
}
