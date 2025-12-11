// text-editor/src/actions/cursor.rs

use std::io::Write as _;

use betterm::scroll;

use crate::Editor;
use crate::to::{ToMaxWith, ToMinWith};
use crate::utf8::Utf8Len;


pub fn move_up(editor: &mut Editor) {
    if editor.cursor.y == 0 {
        if editor.cursor.x == 0 {
            editor.cursor.last_x = 0;
            return;
        }

        editor.cursor.x      = 0;
        editor.cursor.last_x = editor.cursor.x;
    } else {
        editor.cursor.y -= 1;

        editor.cursor.x
            .to_max_with(editor.cursor.last_x)
            .to_min_with(editor.lines[editor.cursor.y].utf8_len());

        if editor.cursor.y + 1 - editor.scroll.y == 0 {
            editor.scroll.y -= 1;

            write!(editor.stdout, "{}", scroll::DOWN).unwrap();
            editor.refresh();

            return;
        }
    }

    write!(
        editor.stdout,
        "{}",
        editor.update_cursor_position()
    ).unwrap();

    editor.stdout.flush().unwrap();
}

pub fn move_down(editor: &mut Editor) {
    let current_line_len = editor.lines[editor.cursor.y].utf8_len();

    if editor.cursor.y == editor.lines.len() - 1 {
        if editor.cursor.x == current_line_len {
            editor.cursor.last_x = current_line_len;
            return;
        }

        editor.cursor.x      = editor.lines[editor.cursor.y].utf8_len();
        editor.cursor.last_x = editor.cursor.x;
    } else {
        editor.cursor.y += 1;

        editor.cursor.x
            .to_max_with(editor.cursor.last_x)
            .to_min_with(editor.lines[editor.cursor.y].utf8_len());

        if editor.cursor.y - 1 - editor.scroll.y == editor.terminal.height - 1 {
            editor.scroll.y += 1;

            write!(editor.stdout, "{}", scroll::UP).unwrap();
            editor.refresh();

            return;
        }
    }

    write!(
        editor.stdout,
        "{}",
        editor.update_cursor_position()
    ).unwrap();

    editor.stdout.flush().unwrap();
}

pub fn move_left(editor: &mut Editor) {
    if editor.cursor.x == 0 {
        if editor.cursor.y == 0 {
            editor.cursor.last_x = 0;
            return;
        }

        editor.cursor.y -= 1;
        editor.cursor.x  = editor.lines[editor.cursor.y].utf8_len();

        if editor.cursor.y + 1 - editor.scroll.y == 0 {
            editor.scroll.y -= 1;

            write!(editor.stdout, "{}", scroll::DOWN).unwrap();
            editor.refresh();

            return;
        }
    } else {
        editor.cursor.x -= 1;
    }

    editor.cursor.last_x = editor.cursor.x;

    write!(
        editor.stdout,
        "{}",
        editor.update_cursor_position()
    ).unwrap();

    editor.stdout.flush().unwrap();
}

pub fn move_right(editor: &mut Editor) {
    let current_line_len = editor.lines[editor.cursor.y].utf8_len();

    if editor.cursor.x == current_line_len {
        if editor.cursor.y == editor.lines.len() - 1 {
            editor.cursor.last_x = current_line_len;
            return;
        }

        editor.cursor.y      += 1;
        editor.cursor.x       = 0;
        editor.cursor.last_x  = 0;

        if editor.cursor.y - 1 - editor.scroll.y == editor.terminal.height - 1 {
            editor.scroll.y += 1;

            write!(editor.stdout, "{}", scroll::UP).unwrap();
            editor.refresh();

            return;
        }
    } else {
        editor.cursor.x      += 1;
        editor.cursor.last_x  = editor.cursor.x;
    }

    write!(
        editor.stdout,
        "{}",
        editor.update_cursor_position()
    ).unwrap();

    editor.stdout.flush().unwrap();
}

pub fn move_to_previous_word(editor: &mut Editor) {
    if editor.cursor.x == 0 {
        move_left(editor);
        return;
    }

    editor.cursor.x      = editor.utf8_position_of_left_whitespace();
    editor.cursor.last_x = editor.cursor.x;

    write!(
        editor.stdout,
        "{}",
        editor.update_cursor_position()
    ).unwrap();

    editor.stdout.flush().unwrap();
}

pub fn move_to_next_word(editor: &mut Editor) {
    if editor.cursor.x == editor.lines[editor.cursor.y].utf8_len() {
        move_right(editor);
        return;
    }

    editor.cursor.x      += editor.utf8_offset_to_right_whitespace();
    editor.cursor.last_x  = editor.cursor.x;

    write!(
        editor.stdout,
        "{}",
        editor.update_cursor_position()
    ).unwrap();

    editor.stdout.flush().unwrap();
}

pub fn move_to_start_of_line(editor: &mut Editor) {
    editor.cursor.last_x = 0;

    if editor.cursor.x == 0 {
        return;
    }

    editor.cursor.x = 0;

    write!(
        editor.stdout,
        "{}",
        editor.update_cursor_position()
    ).unwrap();

    editor.stdout.flush().unwrap();
}

pub fn move_to_end_of_line(editor: &mut Editor) {
    editor.cursor.last_x = editor.lines[editor.cursor.y].utf8_len();

    if editor.cursor.x == editor.cursor.last_x {
        return;
    }

    editor.cursor.x = editor.cursor.last_x;

    write!(
        editor.stdout,
        "{}",
        editor.update_cursor_position()
    ).unwrap();

    editor.stdout.flush().unwrap();
}

pub fn move_to_start_of_file(editor: &mut Editor) {
    if editor.cursor.y == 0 {
        return;
    }

    editor.cursor.y = 0;

    editor.cursor.x
        .to_max_with(editor.cursor.last_x)
        .to_min_with(editor.lines[editor.cursor.y].utf8_len());

    write!(
        editor.stdout,
        "{}",
        editor.update_cursor_position()
    ).unwrap();

    editor.stdout.flush().unwrap();
}

pub fn move_to_end_of_file(editor: &mut Editor) {
    let last_line_i = editor.lines.len() - 1;

    if editor.cursor.y == last_line_i {
        return;
    }

    editor.cursor.y = last_line_i;

    editor.cursor.x
        .to_max_with(editor.cursor.last_x)
        .to_min_with(editor.lines[editor.cursor.y].utf8_len());

    write!(
        editor.stdout,
        "{}",
        editor.update_cursor_position()
    ).unwrap();

    editor.stdout.flush().unwrap();
}

