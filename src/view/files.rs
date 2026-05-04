// mochou-p/text-editor/src/view/files.rs

use std::path::PathBuf;
use super::editing::Editing;
use super::{View, ViewData};
use crate::{Editor, InsertSet};


pub struct Files {
    view_data: ViewData,
    file:      Option<PathBuf>,
    // TODO: keep insertion order
    files:     InsertSet<PathBuf>
}

impl Files {
    pub fn new(editor: &mut Editor) -> Self {
        Self {
            view_data: ViewData::above::<Editing>(editor, 1),
            file:      None,
            files:     InsertSet::new()
        }
    }

    pub fn add_file(&mut self, path: PathBuf) {
        self.files.insert(path.clone());
        self.file = Some(path);
    }
}

impl View for Files {
    fn any          (&mut self) -> &mut dyn std::any::Any { self                  }
    fn name         (         ) ->          String        { String::from("files") }
    fn view_data    (&    self) -> &        ViewData      { &    self.view_data   }
    fn view_data_mut(&mut self) -> &mut     ViewData      { &mut self.view_data   }

    fn print_line(&mut self, editor: &mut Editor, buffer: &mut String, _loop_i: usize, _scrolled_i: usize) {
        let mut size = self.size().x as usize;

        for file in self.files.iter() {
            let text = format!(" {:?} ", file.file_name().unwrap());
            let len  = text.len();

            if len > size {
                break;
            }

            size -= len;

            let color = if let Some(opened) = self.file.as_ref().cloned() {
                if *file == opened {
                    &editor.theme.backgrounds.primary.normal
                } else {
                    &editor.theme.backgrounds.primary.disabled
                }
            } else {
                &editor.theme.backgrounds.primary.disabled
            };

            buffer.push_str(&format!("{color}{text}"));
        }

        buffer.push_str(&format!(
            "{}{}",
            editor.theme.backgrounds.primary.disabled,
            " ".repeat(size)
        ));
    }
}
