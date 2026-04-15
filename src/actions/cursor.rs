// mochou-p/text-editor/src/actions/cursor.rs

use crate::Editor;
use crate::utils::{self, ToWith, Utf8, word};


pub fn line_start(editor: &mut Editor) {
    let file = editor.files.get_mut(&editor.view.file).unwrap();

    for (i, cursor) in file.cursors.iter_mut().enumerate() {
        cursor.x      = 0;
        cursor.last_x = 0;

        if i == 0 {
            Editor::snap_to_cursor(&mut editor.view, cursor);
        }
    }
}

pub fn line_end(editor: &mut Editor) {
    let file = editor.files.get_mut(&editor.view.file).unwrap();

    for (i, cursor) in file.cursors.iter_mut().enumerate() {
        cursor.x      = file.lines[cursor.y as usize].utf8_len();
        cursor.last_x = isize::MAX;

        if i == 0 {
            Editor::snap_to_cursor(&mut editor.view, cursor);
        }
    }
}

pub fn file_start(editor: &mut Editor) {
    let file = editor.files.get_mut(&editor.view.file).unwrap();

    for (i, cursor) in file.cursors.iter_mut().enumerate() {
        if cursor.y != 0 {
            cursor.y             = 0;
            editor.view.scroll.y = 0;

            cursor.x
                .to_max_with(cursor.last_x)
                .to_min_with(file.lines[cursor.y as usize].utf8_len());
        }

        if i == 0 {
            Editor::snap_to_cursor(&mut editor.view, cursor);
        }
    }
}

pub fn file_end(editor: &mut Editor) {
    let file       = editor.files.get_mut(&editor.view.file).unwrap();
    let line_count = file.lines.len() as isize;

    for (i, cursor) in file.cursors.iter_mut().enumerate() {
        if cursor.y != line_count - 1 {
            cursor.y             = line_count - 1;
            editor.view.scroll.y = line_count.saturating_sub(editor.view.size.y);

            cursor.x
                .to_max_with(cursor.last_x)
                .to_min_with(file.lines[cursor.y as usize].utf8_len());
        }

        if i == 0 {
            Editor::snap_to_cursor(&mut editor.view, cursor);
        }
    }
}

pub fn up(editor: &mut Editor) {
    let file = editor.files.get_mut(&editor.view.file).unwrap();

    for (i, cursor) in file.cursors.iter_mut().enumerate() {
        if cursor.y == 0 {
            cursor.x      = 0;
            cursor.last_x = 0;
        } else {
            if i == 0 && cursor.y - editor.view.scroll.y == 0 {
                editor.view.scroll.y -= 1;
            }

            cursor.y -= 1;
            cursor.x
                .to_max_with(cursor.last_x)
                .to_min_with(file.lines[cursor.y as usize].utf8_len());
        }

        if i == 0 {
            Editor::snap_to_cursor(&mut editor.view, cursor);
        }
    }
}

pub fn down(editor: &mut Editor) {
    let file = editor.files.get_mut(&editor.view.file).unwrap();

    for (i, cursor) in file.cursors.iter_mut().enumerate() {
        if cursor.y == (file.lines.len() - 1) as isize {
            cursor.x      = file.lines[cursor.y as usize].utf8_len();
            cursor.last_x = cursor.x;
        } else {
            if i == 0 && cursor.y - editor.view.scroll.y == editor.view.size.y - 1 {
                editor.view.scroll.y += 1;
            }

            cursor.y += 1;
            cursor.x
                .to_max_with(cursor.last_x)
                .to_min_with(file.lines[cursor.y as usize].utf8_len());
        }

        if i == 0 {
            Editor::snap_to_cursor(&mut editor.view, cursor);
        }
    }
}

pub fn left(editor: &mut Editor) {
    let file = editor.files.get_mut(&editor.view.file).unwrap();

    for (i, cursor) in file.cursors.iter_mut().enumerate() {
        if cursor.x == 0 {
            if cursor.y != 0 {
                cursor.y -= 1;
                cursor.x  = file.lines[cursor.y as usize].utf8_len();
            }
        } else {
            cursor.x -= 1;
        }

        cursor.last_x = cursor.x;

        if i == 0 {
            Editor::snap_to_cursor(&mut editor.view, cursor);
        }
    }
}

pub fn right(editor: &mut Editor) {
    let file = editor.files.get_mut(&editor.view.file).unwrap();

    for (i, cursor) in file.cursors.iter_mut().enumerate() {
        if cursor.x == file.lines[cursor.y as usize].utf8_len() {
            if cursor.y != (file.lines.len() - 1) as isize {
                cursor.x  = 0;
                cursor.y += 1;
            }
        } else {
            cursor.x += 1;
        }

        cursor.last_x = cursor.x;

        if i == 0 {
            Editor::snap_to_cursor(&mut editor.view, cursor);
        }
    }
}

pub fn prev_word(editor: &mut Editor) {
    let file = editor.files.get_mut(&editor.view.file).unwrap();

    for (i, cursor) in file.cursors.iter_mut().enumerate() {
        if cursor.x == 0 {
            left(editor);
            return;
        }

        let line  = &file.lines[cursor.y as usize];
        let start = line.chars().nth((cursor.x - 1) as usize).unwrap();

        let end = if utils::is_alphanumericx(start) {
            word::to_left(line, cursor.x, |ch| !utils::is_alphanumericx(ch))
        } else {
            word::to_left(line, cursor.x, |ch| ch != start)
        };

        cursor.x      = end.map(|i| i+1).unwrap_or(0);
        cursor.last_x = cursor.x;

        if i == 0 {
            Editor::snap_to_cursor(&mut editor.view, cursor);
        }
    }
}

pub fn next_word(editor: &mut Editor) {
    let file = editor.files.get_mut(&editor.view.file).unwrap();

    for (i, cursor) in file.cursors.iter_mut().enumerate() {
        let line = &file.lines[cursor.y as usize];

        if cursor.x == line.utf8_len() {
            right(editor);
            return;
        }

        let start = line.chars().nth(cursor.x as usize).unwrap();

        let end = if utils::is_alphanumericx(start) {
            word::to_right(line, cursor.x, |ch| !utils::is_alphanumericx(ch))
        } else {
            word::to_right(line, cursor.x, |ch| ch != start)
        };

        cursor.x      = end.unwrap_or(file.lines[cursor.y as usize].utf8_len());
        cursor.last_x = cursor.x;

        if i == 0 {
            Editor::snap_to_cursor(&mut editor.view, cursor);
        }
    }
}

