// text-editor/src/events.rs

use termion::event::{Event, Key};


pub const CTRL_BACKSPACE: Event = Event::Key(Key::Ctrl('h'));

pub fn ctrl_delete() -> Event {
    Event::Unsupported(vec![27, 91, 51, 59, 53, 126])
}

