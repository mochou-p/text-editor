// text-editor/src/actions/typing.rs

use std::io::Write as _;

use betterm::clear;

use crate::utf8::{Utf8Len, Utf8Range, Utf8Drain};

use crate::Editor;


pub fn erase_character_left(editor: &mut Editor) {
    if editor.cursor.x == 0 {
        if editor.cursor.y == 0 {
            editor.cursor.last_x = 0;
            return;
        }

        editor.wrapping_erase_character_left();
    } else {
        editor.normal_erase_character_left();
    }

    editor.stdout.flush().unwrap();

    editor.refresh();
}

pub fn erase_word_left(editor: &mut Editor) {
    if editor.cursor.x == 0 {
        if editor.cursor.y == 0 {
            editor.cursor.last_x = 0;
            return;
        }

        editor.wrapping_erase_character_left();
    } else {
        let start = editor.utf8_position_of_left_whitespace();

        editor.lines[editor.cursor.y]
            .utf8_drain(start, editor.cursor.x);

        editor.cursor.x      = start;
        editor.cursor.last_x = start;

        write!(
            editor.stdout,
            "{}{}{}{}",
            editor.update_cursor_position(),
            editor.lines[editor.cursor.y].utf8_range(editor.cursor.x, editor.lines[editor.cursor.y].utf8_len()),
            clear::LINE_RIGHT_OF_CURSOR,
            editor.update_cursor_position()
        ).unwrap();
    }

    editor.stdout.flush().unwrap();
}

pub fn erase_character_right(editor: &mut Editor) {
    let current_line_len = editor.lines[editor.cursor.y].utf8_len();

    if editor.cursor.x == current_line_len {
        if editor.cursor.y == editor.lines.len() - 1 {
            editor.cursor.last_x = current_line_len;
            return;
        }

        editor.wrapping_erase_character_right();
    } else {
        editor.normal_erase_character_right();
    }

    editor.stdout.flush().unwrap();

    editor.refresh();
}

pub fn erase_word_right(editor: &mut Editor) {
    let current_line_len = editor.lines[editor.cursor.y].utf8_len();

    if editor.cursor.x == current_line_len {
        if editor.cursor.y == editor.lines.len() - 1 {
            editor.cursor.last_x = current_line_len;
            return;
        }

        editor.wrapping_erase_character_right();
    } else {
        let offset = editor.utf8_offset_to_right_whitespace();

        editor.lines[editor.cursor.y]
            .utf8_drain(editor.cursor.x, editor.cursor.x + offset);

        editor.cursor.last_x = editor.cursor.x;

        write!(
            editor.stdout,
            "{}{}{}",
            editor.lines[editor.cursor.y].utf8_range(editor.cursor.x, editor.lines[editor.cursor.y].utf8_len()),
            clear::LINE_RIGHT_OF_CURSOR,
            editor.update_cursor_position()
        ).unwrap();
    }

    editor.stdout.flush().unwrap();
}

