// mochou-p/text-editor/src/view/browsing.rs

use std::io::Write as _;
use betterm::cursor;
use super::{View, ViewData};
use crate::Editor;


pub struct Browsing {
    view_data: ViewData
}

impl Browsing {
    pub fn new(editor: &mut Editor) -> Self {
        Self { view_data: ViewData::left_of(editor, "editing", 16) }
    }
}

impl View for Browsing {
    fn name     (         ) ->      String   { String::from("browsing") }
    fn view_data(&mut self) -> &mut ViewData { &mut self.view_data      }

    fn reprint(&mut self, editor: &mut Editor) {
        for i in 0..self.size().y {
            write!(
                editor.stdout,
                "{}{}{}",
                editor.theme.backgrounds.secondary.normal,
                cursor::MoveToColumnAndRow(
                    (self.position().x + 1    ) as u16,
                    (self.position().y + 1 + i) as u16
                ),
                " ".repeat(self.size().x as usize)
            ).unwrap();
        }
    }
}
