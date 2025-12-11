// text-editor/src/actions/mod.rs

mod cursor;
mod editor;
mod file;
mod typing;

use super::Editor;


pub type ActionIndex = usize;
    type ActionFn    = fn(&mut Editor) -> ();

pub const ACTIONS: [ActionFn; 16] = [
    editor::exit,

    file::save,

    cursor::move_left,
    cursor::move_right,
    cursor::move_up,
    cursor::move_down,
    cursor::move_to_previous_word,
    cursor::move_to_next_word,
    cursor::move_to_start_of_line,
    cursor::move_to_end_of_line,
    cursor::move_to_start_of_file,
    cursor::move_to_end_of_file,

    typing::erase_character_left,
    typing::erase_character_right,
    typing::erase_word_left,
    typing::erase_word_right
];

pub struct Action;

// TODO: macro/refactor
impl Action {
    pub const EDITOR_EXIT:                  ActionIndex =  0;

    pub const FILE_SAVE:                    ActionIndex =  1;

    pub const CURSOR_MOVE_LEFT:             ActionIndex =  2;
    pub const CURSOR_MOVE_RIGHT:            ActionIndex =  3;
    pub const CURSOR_MOVE_UP:               ActionIndex =  4;
    pub const CURSOR_MOVE_DOWN:             ActionIndex =  5;
    pub const CURSOR_MOVE_TO_PREVIOUS_WORD: ActionIndex =  6;
    pub const CURSOR_MOVE_TO_NEXT_WORD:     ActionIndex =  7;
    pub const CURSOR_MOVE_TO_START_OF_LINE: ActionIndex =  8;
    pub const CURSOR_MOVE_TO_END_OF_LINE:   ActionIndex =  9;
    pub const CURSOR_MOVE_TO_START_OF_FILE: ActionIndex = 10;
    pub const CURSOR_MOVE_TO_END_OF_FILE:   ActionIndex = 11;

    pub const TYPING_ERASE_CHARACTER_LEFT:  ActionIndex = 12;
    pub const TYPING_ERASE_CHARACTER_RIGHT: ActionIndex = 13;
    pub const TYPING_ERASE_WORD_LEFT:       ActionIndex = 14;
    pub const TYPING_ERASE_WORD_RIGHT:      ActionIndex = 15;
}

