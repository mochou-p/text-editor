// mochou-p/text-editor/src/view/editing/actions/typing.rs

use crate::utils::{self, Utf8, Utf8Mut, word};


impl super::super::Editing {
    pub fn newline(&mut self) {
        let Some(file) = self.file.as_ref() else { return; };
        let      file  = self.files.get_mut(file).unwrap();
        file.clean     = false;

        for cursor in &mut file.cursors {
            let trail = file.lines[cursor.y as usize].utf8_split_off(cursor.x);

            cursor.x       = 0;
            cursor.last_x  = cursor.x;
            cursor.y      += 1;

            file.lines.insert(cursor.y as usize, trail);
        }

        self.snap_to_cursor();
    }

    pub fn tab(&mut self) {
        let Some(file) = self.file.as_ref() else { return; };
        let      file  = self.files.get_mut(file).unwrap();
        file.clean     = false;

        for cursor in &mut file.cursors {
            let count = 4 - (cursor.x as usize % 4);

            file.lines[cursor.y as usize].utf8_insert_str(cursor.x, &(" ".repeat(count)));

            cursor.x += count as isize;
        }

        self.snap_to_cursor();
    }

    pub fn character(&mut self, ch: char) {
        let Some(file) = self.file.as_ref() else { return; };
        let      file  = self.files.get_mut(file).unwrap();
        file.clean     = false;

        for cursor in &mut file.cursors {
            file.lines[cursor.y as usize].utf8_insert(cursor.x, ch);

            cursor.x      += 1;
            cursor.last_x  = cursor.x;
        }

        self.snap_to_cursor();
    }

    pub fn erase_left(&mut self) {
        let Some(file) = self.file.as_ref() else { return; };
        let      file  = self.files.get_mut(file).unwrap();

        for cursor in &mut file.cursors {
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
        }

        self.snap_to_cursor();
    }

    pub fn erase_right(&mut self) {
        let Some(file) = self.file.as_ref() else { return; };
        let      file  = self.files.get_mut(file).unwrap();

        for cursor in &mut file.cursors {
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
        }

        self.snap_to_cursor();
    }

    pub fn move_line_up(&mut self) {
        let Some(file) = self.file.as_ref() else { return; };
        let      file  = self.files.get_mut(file).unwrap();

        for cursor in &mut file.cursors {
            if cursor.y != 0 {
                file.lines.swap(cursor.y as usize, (cursor.y - 1) as usize);
                cursor.y -= 1;

                file.clean = false;
            }

            cursor.last_x = cursor.x;
        }

        self.snap_to_cursor();
    }

    pub fn move_line_down(&mut self) {
        let Some(file) = self.file.as_ref() else { return; };
        let      file  = self.files.get_mut(file).unwrap();

        for cursor in &mut file.cursors {
            if cursor.y != (file.lines.len() - 1) as isize {
                file.lines.swap(cursor.y as usize, (cursor.y + 1) as usize);
                cursor.y += 1;

                file.clean = false;
            }

            cursor.last_x = cursor.x;
        }

        self.snap_to_cursor();
    }

    pub fn erase_prev_word(&mut self) {
        let Some(file) = self.file.as_ref() else { return; };
        let      file  = self.files.get_mut(file).unwrap();

        for cursor in &mut file.cursors {
            if cursor.x == 0 {
                self.erase_left();
                return;
            }

            let line  = &mut file.lines[cursor.y as usize];
            let start = line.chars().nth((cursor.x - 1) as usize).unwrap();

            let end = if utils::is_alphanumericx(start) {
                word::to_left(line, cursor.x, |ch| !utils::is_alphanumericx(ch))
            } else {
                word::to_left(line, cursor.x, |ch| ch != start)
            };

            let old_x     = cursor.x;
            cursor.x      = end.map(|i| i+1).unwrap_or(0);
            cursor.last_x = cursor.x;

            line.utf8_drain(cursor.x, old_x);
        }

        self.snap_to_cursor();
    }

    pub fn erase_next_word(&mut self) {
        let Some(file) = self.file.as_ref() else { return; };
        let      file  = self.files.get_mut(file).unwrap();

        for cursor in &mut file.cursors {
            let line = &file.lines[cursor.y as usize];

            if cursor.x == line.utf8_len() {
                self.erase_right();
                return;
            }

            let line  = &mut file.lines[cursor.y as usize];
            let start = line.chars().nth(cursor.x as usize).unwrap();

            let end = if utils::is_alphanumericx(start) {
                word::to_right(line, cursor.x, |ch| !utils::is_alphanumericx(ch))
            } else {
                word::to_right(line, cursor.x, |ch| ch != start)
            };

            cursor.last_x = cursor.x;

            line.utf8_drain(cursor.x, end.unwrap_or(line.utf8_len()));
        }

        self.snap_to_cursor();
    }
}
