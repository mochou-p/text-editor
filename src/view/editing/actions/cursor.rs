// mochou-p/text-editor/src/view/editing/actions/cursor.rs

use crate::utils::{self, ToWith, Utf8, word};


impl super::super::Editing {
    pub fn line_start(&mut self) {
        let Some(file) = self.file.as_ref() else { return; };
        let      file  = self.files.get_mut(file).unwrap();

        for cursor in &mut file.cursors {
            cursor.x      = 0;
            cursor.last_x = 0;
        }

        self.snap_to_cursor();
    }

    pub fn line_end(&mut self) {
        let Some(file) = self.file.as_ref() else { return; };
        let      file  = self.files.get_mut(file).unwrap();

        for cursor in &mut file.cursors {
            cursor.x      = file.lines[cursor.y as usize].utf8_len();
            cursor.last_x = isize::MAX;
        }

        self.snap_to_cursor();
    }

    pub fn file_start(&mut self) {
        let Some(file) = self.file.as_ref() else { return; };
        let      file  = self.files.get_mut(file).unwrap();

        for cursor in &mut file.cursors {
            if cursor.y != 0 {
                cursor.y            = 0;

                cursor.x
                    .to_max_with(cursor.last_x)
                    .to_min_with(file.lines[cursor.y as usize].utf8_len());
            }
        }

        self.snap_to_cursor();
    }

    pub fn file_end(&mut self) {
        let Some(file) = self.file.as_ref() else { return; };
        let      file  = self.files.get_mut(file).unwrap();
        let line_count = file.lines.len() as isize;

        for cursor in &mut file.cursors {
            if cursor.y != line_count - 1 {
                cursor.y = line_count - 1;

                cursor.x
                    .to_max_with(cursor.last_x)
                    .to_min_with(file.lines[cursor.y as usize].utf8_len());
            }
        }

        self.snap_to_cursor();
    }

    pub fn up(&mut self) {
        let Some(file) = self.file.as_ref() else { return; };
        let      file  = self.files.get_mut(file).unwrap();

        for cursor in &mut file.cursors {
            if cursor.y == 0 {
                cursor.x      = 0;
                cursor.last_x = 0;
            } else {
                cursor.y -= 1;
                cursor.x
                    .to_max_with(cursor.last_x)
                    .to_min_with(file.lines[cursor.y as usize].utf8_len());
            }
        }

        self.snap_to_cursor();
    }

    pub fn down(&mut self) {
        let Some(file) = self.file.as_ref() else { return; };
        let      file  = self.files.get_mut(file).unwrap();

        for cursor in &mut file.cursors {
            if cursor.y == (file.lines.len() - 1) as isize {
                cursor.x      = file.lines[cursor.y as usize].utf8_len();
                cursor.last_x = cursor.x;
            } else {
                cursor.y += 1;
                cursor.x
                    .to_max_with(cursor.last_x)
                    .to_min_with(file.lines[cursor.y as usize].utf8_len());
            }
        }

        self.snap_to_cursor();
    }

    pub fn left(&mut self) {
        let Some(file) = self.file.as_ref() else { return; };
        let      file  = self.files.get_mut(file).unwrap();

        for cursor in &mut file.cursors {
            if cursor.x == 0 {
                if cursor.y != 0 {
                    cursor.y -= 1;
                    cursor.x  = file.lines[cursor.y as usize].utf8_len();
                }
            } else {
                cursor.x -= 1;
            }

            cursor.last_x = cursor.x;
        }

        self.snap_to_cursor();
    }

    pub fn right(&mut self) {
        let Some(file) = self.file.as_ref() else { return; };
        let      file  = self.files.get_mut(file).unwrap();

        for cursor in &mut file.cursors {
            if cursor.x == file.lines[cursor.y as usize].utf8_len() {
                if cursor.y != (file.lines.len() - 1) as isize {
                    cursor.x  = 0;
                    cursor.y += 1;
                }
            } else {
                cursor.x += 1;
            }

            cursor.last_x = cursor.x;
        }

        self.snap_to_cursor();
    }

    pub fn prev_word(&mut self) {
        let Some(file) = self.file.as_ref() else { return; };
        let      file  = self.files.get_mut(file).unwrap();

        for cursor in &mut file.cursors {
            if cursor.x == 0 {
                self.left();
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
        }

        self.snap_to_cursor();
    }

    pub fn next_word(&mut self) {
        let Some(file) = self.file.as_ref() else { return; };
        let      file  = self.files.get_mut(file).unwrap();

        for cursor in &mut file.cursors {
            let line = &file.lines[cursor.y as usize];

            if cursor.x == line.utf8_len() {
                self.right();
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
        }

        self.snap_to_cursor();
    }
}
