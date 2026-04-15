// mochou-p/text-editor/src/actions/typing.rs

use crate::Editor;
use crate::utf8::{Utf8, Utf8Mut};
use crate::utils;


pub fn newline(editor: &mut Editor) {
    let file   = editor.files.get_mut(&editor.view.file).unwrap();
    file.clean = false;

    for (i, cursor) in file.cursors.iter_mut().enumerate() {
        let trail = file.lines[cursor.y as usize].utf8_split_off(cursor.x);

        cursor.x       = 0;
        cursor.last_x  = cursor.x;
        cursor.y      += 1;

        file.lines.insert(cursor.y as usize, trail);

        if i == 0 {
            Editor::snap_to_cursor(&mut editor.view, cursor);
        }
    }
}

pub fn tab(editor: &mut Editor) {
    let file   = editor.files.get_mut(&editor.view.file).unwrap();
    file.clean = false;

    for (i, cursor) in file.cursors.iter_mut().enumerate() {
        let count = 4 - (cursor.x as usize % 4);

        file.lines[cursor.y as usize].utf8_insert_str(cursor.x, &(" ".repeat(count)));

        cursor.x += count as isize;

        if i == 0 {
            Editor::snap_to_cursor(&mut editor.view, cursor);
        }
    }
}

pub fn character(editor: &mut Editor, ch: char) {
    let file   = editor.files.get_mut(&editor.view.file).unwrap();
    file.clean = false;

    for (i, cursor) in file.cursors.iter_mut().enumerate() {
        file.lines[cursor.y as usize].utf8_insert(cursor.x, ch);

        cursor.x      += 1;
        cursor.last_x  = cursor.x;

        if i == 0 {
            Editor::snap_to_cursor(&mut editor.view, cursor);
        }
    }
}

pub fn erase_left(editor: &mut Editor) {
    let file = editor.files.get_mut(&editor.view.file).unwrap();

    for (i, cursor) in file.cursors.iter_mut().enumerate() {
        if cursor.x == 0 {
            if cursor.y != 0 {
                let line = file.lines.remove(cursor.y as usize);

                cursor.y -= 1;
                cursor.x  = file.lines[cursor.y as usize].utf8_len();

                file.lines[cursor.y as usize].push_str(&line);
                file.clean = false;
            }
        } else {
            cursor.x -= 1;

            file.lines[cursor.y as usize].utf8_remove(cursor.x);
            file.clean = false;
        }

        cursor.last_x = cursor.x;

        if i == 0 {
            Editor::snap_to_cursor(&mut editor.view, cursor);
        }
    }
}

pub fn erase_right(editor: &mut Editor) {
    let file = editor.files.get_mut(&editor.view.file).unwrap();

    for (i, cursor) in file.cursors.iter_mut().enumerate() {
        if cursor.x == file.lines[cursor.y as usize].utf8_len() {
            if cursor.y != (file.lines.len() - 1) as isize {
                let line = file.lines.remove((cursor.y + 1) as usize);

                file.lines[cursor.y as usize].push_str(&line);
                file.clean = false;
            }
        } else {
            file.lines[cursor.y as usize].utf8_remove(cursor.x);
            file.clean = false;
        }

        cursor.last_x = cursor.x;

        if i == 0 {
            Editor::snap_to_cursor(&mut editor.view, cursor);
        }
    }
}

pub fn move_line_up(editor: &mut Editor) {
    let file = editor.files.get_mut(&editor.view.file).unwrap();

    for (i, cursor) in file.cursors.iter_mut().enumerate() {
        if cursor.y != 0 {
            file.lines.swap(cursor.y as usize, (cursor.y - 1) as usize);
            cursor.y -= 1;

            file.clean = false;
        }

        cursor.last_x = cursor.x;

        if i == 0 {
            Editor::snap_to_cursor(&mut editor.view, cursor);
        }
    }
}

pub fn move_line_down(editor: &mut Editor) {
    let file = editor.files.get_mut(&editor.view.file).unwrap();

    for (i, cursor) in file.cursors.iter_mut().enumerate() {
        if cursor.y != (file.lines.len() - 1) as isize {
            file.lines.swap(cursor.y as usize, (cursor.y + 1) as usize);
            cursor.y += 1;

            file.clean = false;
        }

        cursor.last_x = cursor.x;

        if i == 0 {
            Editor::snap_to_cursor(&mut editor.view, cursor);
        }
    }
}

pub fn erase_prev_word(editor: &mut Editor) {
    let file = editor.files.get_mut(&editor.view.file).unwrap();

    for (i, cursor) in file.cursors.iter_mut().enumerate() {
        if cursor.x == 0 {
            erase_left(editor);
            return;
        }

        let line  = &mut file.lines[cursor.y as usize];
        let start = line.chars().nth((cursor.x - 1) as usize).unwrap();

        let end = if utils::is_alphanumericx(start) {
            utils::find_to_left(line, cursor.x, |ch| !utils::is_alphanumericx(ch))
        } else {
            utils::find_to_left(line, cursor.x, |ch| ch != start)
        };

        let old_x     = cursor.x;
        cursor.x      = end.map(|i| i+1).unwrap_or(0);
        cursor.last_x = cursor.x;

        line.utf8_drain(cursor.x, old_x);

        if i == 0 {
            Editor::snap_to_cursor(&mut editor.view, cursor);
        }
    }
}

pub fn erase_next_word(editor: &mut Editor) {
    let file = editor.files.get_mut(&editor.view.file).unwrap();

    for (i, cursor) in file.cursors.iter_mut().enumerate() {
        let line = &file.lines[cursor.y as usize];

        if cursor.x == line.utf8_len() {
            erase_right(editor);
            return;
        }

        let line  = &mut file.lines[cursor.y as usize];
        let start = line.chars().nth(cursor.x as usize).unwrap();

        let end = if utils::is_alphanumericx(start) {
            utils::find_to_right(line, cursor.x, |ch| !utils::is_alphanumericx(ch))
        } else {
            utils::find_to_right(line, cursor.x, |ch| ch != start)
        };

        cursor.last_x = cursor.x;

        line.utf8_drain(cursor.x, end.unwrap_or(line.utf8_len()));

        if i == 0 {
            Editor::snap_to_cursor(&mut editor.view, cursor);
        }
    }
}

