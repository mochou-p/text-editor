// text-editor/src/keybinds.rs

use std::collections::HashMap;

use termion::event::{Event, Key};

use super::actions::{Action, ActionIndex};
use super::events;


pub struct Keybinds(pub HashMap<Event, ActionIndex>);

impl Default for Keybinds {
    fn default() -> Self {
        Self(HashMap::from([
            (Event::Key(Key::Esc      ), Action::EDITOR_EXIT                 ),

            (Event::Key(Key::Left     ), Action::CURSOR_MOVE_LEFT            ),
            (Event::Key(Key::Right    ), Action::CURSOR_MOVE_RIGHT           ),
            (Event::Key(Key::Up       ), Action::CURSOR_MOVE_UP              ),
            (Event::Key(Key::Down     ), Action::CURSOR_MOVE_DOWN            ),
            (Event::Key(Key::CtrlLeft ), Action::CURSOR_MOVE_TO_PREVIOUS_WORD),
            (Event::Key(Key::CtrlRight), Action::CURSOR_MOVE_TO_NEXT_WORD    ),
            (Event::Key(Key::Home     ), Action::CURSOR_MOVE_TO_START_OF_LINE),
            (Event::Key(Key::End      ), Action::CURSOR_MOVE_TO_END_OF_LINE  ),
            (Event::Key(Key::CtrlHome ), Action::CURSOR_MOVE_TO_START_OF_FILE),
            (Event::Key(Key::CtrlEnd  ), Action::CURSOR_MOVE_TO_END_OF_FILE  ),

            (Event::Key(Key::Backspace), Action::TYPING_ERASE_CHARACTER_LEFT ),
            (Event::Key(Key::Delete   ), Action::TYPING_ERASE_CHARACTER_RIGHT),
            (events::CTRL_BACKSPACE    , Action::TYPING_ERASE_WORD_LEFT      ),
            (events::ctrl_delete()     , Action::TYPING_ERASE_WORD_RIGHT     )
        ]))
    }
}

