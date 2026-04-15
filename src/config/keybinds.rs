// mochou-p/text-editor/src/config/keybinds.rs

use std::collections::HashMap;

use termion::event::{Event, Key};

use crate::Editor;
use crate::actions::{cursor, editor, file, typing};


type Action = fn(&mut Editor);

pub struct Keybinds(pub HashMap<Event, Action>);

impl Default for Keybinds {
    fn default() -> Self {
        Self(HashMap::from([
            (Event::Key(Key::Esc           ), editor::exit as Action ),
            (Event::Key(Key::Ctrl('s')     ), file  ::save           ),
            (Event::Key(Key::Left          ), cursor::left           ),
            (Event::Key(Key::Right         ), cursor::right          ),
            (Event::Key(Key::Up            ), cursor::up             ),
            (Event::Key(Key::Down          ), cursor::down           ),
            (Event::Key(Key::CtrlLeft      ), cursor::prev_word      ),
            (Event::Key(Key::CtrlRight     ), cursor::next_word      ),
            (Event::Key(Key::Home          ), cursor::line_start     ),
            (Event::Key(Key::End           ), cursor::line_end       ),
            (Event::Key(Key::CtrlHome      ), cursor::file_start     ),
            (Event::Key(Key::CtrlEnd       ), cursor::file_end       ),
            (Event::Key(Key::Char('\n')    ), typing::newline        ),
            (Event::Key(Key::Char('\t')    ), typing::tab            ),
            (Event::Key(Key::Backspace     ), typing::erase_left     ),
            (Event::Key(Key::Delete        ), typing::erase_right    ),
            (                CTRL_BACKSPACE , typing::erase_prev_word),
            (                ctrl_delete()  , typing::erase_next_word),
            (Event::Key(Key::AltUp         ), typing::move_line_up   ),
            (Event::Key(Key::AltDown       ), typing::move_line_down )
        ]))
    }
}

const CTRL_BACKSPACE: Event = Event::Key(Key::Ctrl('h'));

fn ctrl_delete() -> Event {
    Event::Unsupported(vec![27, 91, 51, 59, 53, 126])
}

