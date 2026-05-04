// mochou-p/text-editor/src/view/mod.rs

mod browsing;
mod editing;
mod files;

use termion::event::Event;
use crate::{Editor, Ivec2};

pub use {browsing::Browsing, editing::Editing, files::Files};


pub trait View {
    fn any          (&mut self) -> &mut dyn std::any::Any           ;
    fn name         (         ) ->          String where Self: Sized;
    fn view_data    (&    self) -> &        ViewData                ;
    fn view_data_mut(&mut self) -> &mut     ViewData                ;

    fn position    (&    self) ->      Ivec2 {      self.view_data    ().position }
    fn position_mut(&mut self) -> &mut Ivec2 { &mut self.view_data_mut().position }
    fn     size    (&    self) ->      Ivec2 {      self.view_data    ().size     }
    fn     size_mut(&mut self) -> &mut Ivec2 { &mut self.view_data_mut().size     }
    fn   scroll    (&    self) ->      Ivec2 {      self.view_data    ().scroll   }
    fn   scroll_mut(&mut self) -> &mut Ivec2 { &mut self.view_data_mut().scroll   }

    fn print_line(&mut self, editor: &mut Editor, buffer: &mut String, _loop_i: usize, _scrolled_i: usize) {
        buffer.push_str(&format!(
            "{}{}",
            editor.theme.special.error,
            " ".repeat(self.size().x as usize)
        ));
    }

    fn handle_event(&mut self, _editor: &mut Editor, _event: Event) {}
}

// NOTE: i know these dont make sense signed, its temp
#[derive(Default)]
pub struct ViewData {
    pub position: Ivec2,
    pub size:     Ivec2,
    pub scroll:   Ivec2
}

#[allow(dead_code)]
impl ViewData {
    fn fullscreen() -> Self {
        let position = Ivec2::ZERO;
        let size     = Ivec2::from(termion::terminal_size().unwrap());
        let scroll   = Ivec2::ZERO;

        Self { position, size, scroll }
    }

    fn left_of<T: View + 'static>(editor: &mut Editor, width: isize) -> Self {
        editor.view::<T, Self>(|_, other| {
            let position = other.position();
            let size     = Ivec2 { x: width, y: other.size().y };
            let scroll   = Ivec2::ZERO;

            other.position_mut().x += width;
            other.    size_mut().x -= width;

            Self { position, size, scroll }
        })
    }

    fn right_of<T: View + 'static>(editor: &mut Editor, width: isize) -> Self {
        editor.view::<T, Self>(|_, other| {
            let position = Ivec2 {
                x: other.position().x + other.size().x - width,
                y: other.position().y
            };
            let size     = Ivec2 { x: width, y: other.size().y };
            let scroll   = Ivec2::ZERO;

            other.size_mut().x -= width;

            Self { position, size, scroll }
        })
    }

    fn above<T: View + 'static>(editor: &mut Editor, height: isize) -> Self {
        editor.view::<T, Self>(|_, other| {
            let position = other.position();
            let size     = Ivec2 { x: other.size().x, y: height };
            let scroll   = Ivec2::ZERO;

            other.position_mut().y += height;
            other.    size_mut().y -= height;

            Self { position, size, scroll }
        })
    }

    fn under<T: View + 'static>(editor: &mut Editor, height: isize) -> Self {
        editor.view::<T, Self>(|_, other| {
            let position = Ivec2 {
                x: other.position().x,
                y: other.position().y + other.size().y - height
            };
            let size     = Ivec2 { x: other.size().x, y: height };
            let scroll   = Ivec2::ZERO;

            other.size_mut().x -= height;

            Self { position, size, scroll }
        })
    }
}
