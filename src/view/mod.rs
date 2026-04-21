// mochou-p/text-editor/src/view/mod.rs

mod browsing;
mod editing;
mod files;

use termion::event::Event;
use crate::{Editor, Ivec2};

pub use {browsing::Browsing, editing::Editing, files::Files};


pub trait View {
    fn name     (                              ) ->      String where Self: Sized;
    fn view_data(&mut self                     ) -> &mut ViewData                ;
    fn reprint  (&mut self, editor: &mut Editor)                                 ;

    fn position(&mut self) -> &mut Ivec2 { &mut self.view_data().position }
    fn size    (&mut self) -> &mut Ivec2 { &mut self.view_data().size     }

    fn handle_event(&mut self, _editor: &mut Editor, _event: Event) {}
}

#[derive(Default)]
pub struct ViewData {
    pub position: Ivec2,
    pub size:     Ivec2
}

#[allow(dead_code)]
impl ViewData {
    fn fullscreen() -> Self {
        let position = Ivec2::ZERO;
        let size     = Ivec2::from(termion::terminal_size().unwrap());

        Self { position, size }
    }

    fn left_of(editor: &mut Editor, name: &str, width: isize) -> Self {
        let other = editor.views.get_mut(name).unwrap();

        let position = *other.position();
        let size     = Ivec2 { x: width, y: other.size().y };

        other.position().x += width;
        other.size()    .x -= width;

        Self { position, size }
    }

    fn right_of(editor: &mut Editor, name: &str, width: isize) -> Self {
        let other = editor.views.get_mut(name).unwrap();

        let position = Ivec2 {
            x: other.position().x + other.size().x - width,
            y: other.position().y
        };
        let size = Ivec2 { x: width, y: other.size().y };

        other.size().x -= width;

        Self { position, size }
    }

    fn above(editor: &mut Editor, name: &str, height: isize) -> Self {
        let other = editor.views.get_mut(name).unwrap();

        let position = *other.position();
        let size     = Ivec2 { x: other.size().x, y: height };

        other.position().y += height;
        other.size()    .y -= height;

        Self { position, size }
    }

    fn under(editor: &mut Editor, name: &str, height: isize) -> Self {
        let other = editor.views.get_mut(name).unwrap();

        let position = Ivec2 {
            x: other.position().x,
            y: other.position().y + other.size().y - height
        };
        let size = Ivec2 { x: other.size().x, y: height };

        other.size().x -= height;

        Self { position, size }
    }
}
