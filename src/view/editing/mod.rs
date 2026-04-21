// mochou-p/text-editor/src/view/editing/mod.rs

mod actions;

use std::io::Write as _;
use termion::event::{Event, Key, MouseButton, MouseEvent};
use betterm::cursor as bcursor;
use super::{View, ViewData};
use crate::{Editor, Ivec2, Cursor};
use crate::utils::{ToWith, Utf8};


pub struct Editing {
    view_data: ViewData,
    file:      Option<String>,
    scroll:    Ivec2
}

impl Editing {
    pub fn new(editor: &mut Editor) -> Self {
        let file_option = std::env::args().nth(1);

        if let Some(ref file) = file_option {
            let     string = std::fs::read_to_string(&file).unwrap();
            let mut lines  = string.lines().map(str::to_owned).collect::<Vec<String>>();

            if lines.is_empty() || string.ends_with('\n') {
                lines.push(String::new());
            }

            editor.files.insert(file.to_owned(), crate::File {
                clean:   true,
                cursors: vec![Cursor { last_x: 0, x: 0, y: 0 }],
                lines
            });
        }

        Self {
            view_data: ViewData::fullscreen(),
            file:      file_option,
            scroll:    Ivec2::ZERO
        }
    }

    fn cursor_visible_relative_position(&self, editor: &Editor, file: &String) -> (isize, isize) {
        let cursor = &editor.files[file].cursors[0];

        let x = cursor.x + 1 - self.scroll.x;
        let y = cursor.y + 1 - self.scroll.y;

        (x, y)
    }

    fn snap_to_cursor(&mut self, cursor: &Cursor) {
        if cursor.y < self.scroll.y {
            self.scroll.y = cursor.y;
        } else if cursor.y > self.scroll.y + self.size().y - 1 {
            self.scroll.y = cursor.y - self.size().y + 1;
        }

        if cursor.x < self.scroll.x {
            self.scroll.x = cursor.x;
        } else if cursor.x > self.scroll.x + self.size().x - 1 {
            self.scroll.x = cursor.x - self.size().x + 1;
        }
    }

    // TODO: wrong
    fn warp_cursor(&mut self, editor: &mut Editor, x: u16, y: u16) {
        let Some(ref file) = self.file.as_ref().cloned() else { return; };

        let y = {
            let line_count = editor.files[file].lines.len() as isize;
            let file       = &mut editor.files.get_mut(file).unwrap();

            file.cursors.drain(1..);

            let cursor = &mut file.cursors[0];

            cursor.y = (y - 1) as isize - self.position().y + self.scroll.y;
            cursor.x = (x - 1) as isize - self.position().x + self.scroll.x;

            cursor.y.to_min_with(line_count - 1);
            cursor.y as usize
        };

        let line_len = editor.files[file].lines[y].utf8_len();
        let cursor   = &mut editor.files.get_mut(file).unwrap().cursors[0];

        cursor.x.to_min_with(line_len);
        cursor.last_x = (x - 1) as isize;
    }
}

impl View for Editing {
    fn name     (         ) ->      String   { String::from("editing") }
    fn view_data(&mut self) -> &mut ViewData { &mut self.view_data     }

    fn reprint(&mut self, editor: &mut Editor) {
        let Some(file) = self.file.as_ref().cloned() else {
            for i in 0..self.size().y {
                write!(
                    editor.stdout,
                    "{}{}{}",
                    editor.theme.backgrounds.primary.disabled,
                    bcursor::MoveToColumnAndRow(
                        (self.position().x + 1    ) as u16,
                        (self.position().y + 1 + i) as u16
                    ),
                    " ".repeat(self.size().x as usize)
                ).unwrap();
            }

            return;
        };

        for i in 0..self.size().y {
            let x = self.scroll.x;
            let y = self.scroll.y + i;

            if y < editor.files[&file].lines.len() as isize {
                let line         = &editor.files[&file].lines[y as usize];
                let visible_line = line.utf8_range(x, x + self.size().x);
                let cursor_line  = y == editor.files[&file].cursors[0].y;

                let style = if cursor_line {
                    (&editor.theme.backgrounds.primary.active, &editor.theme.foreground.active)
                } else {
                    (&editor.theme.backgrounds.primary.normal, &editor.theme.foreground.normal)
                };

                write!(
                    editor.stdout,
                    "{}{}{}{visible_line}{}",
                    style.0,
                    style.1,
                    bcursor::MoveToColumnAndRow(
                        (self.position().x + 1    ) as u16,
                        (self.position().y + 1 + i) as u16
                    ),
                    " ".repeat((self.size().x - visible_line.utf8_len()).max(0) as usize)
                ).unwrap();

                // TODO: this can be simplified a lot
                if !cursor_line && line.utf8_len() > self.scroll.x && line.ends_with(' ') {
                    let line_len = line.utf8_len() as usize;
                    let count    = line.rfind(|ch| ch != ' ')
                        .map(|n| line_len - n - 1)
                        .unwrap_or(line_len);

                    let start = (line_len - count) as isize;
                    if start < self.scroll.x + self.size().x {
                        let real_start    = self.position().x + 1 + start - self.scroll.x;
                        let left_overflow = (real_start - self.position().x - 1).min(0);

                        write!(
                            editor.stdout,
                            "{}{}{}",
                            bcursor::MoveToColumn((real_start - left_overflow) as u16),
                            editor.theme.special.error,
                            " ".repeat(
                                (
                                    count.min((self.size().x - (start - self.scroll.x)) as usize) as isize
                                    -
                                    (-left_overflow)
                                ) as usize
                            )
                        ).unwrap();
                    }
                }

                if self.scroll.x > 0 {
                    write!(
                        editor.stdout,
                        "{}{}<",
                        bcursor::MoveToColumn((self.position().x + 1) as u16),
                        editor.theme.special.overflow
                    ).unwrap();
                }

                if line.utf8_len() - self.scroll.x - self.size().x > 0 {
                    write!(
                        editor.stdout,
                        "{}{}>",
                        bcursor::MoveToColumn((self.position().x + self.size().x) as u16),
                        editor.theme.special.overflow
                    ).unwrap();
                }
            } else {
                write!(
                    editor.stdout,
                    "{}{}{}",
                    editor.theme.backgrounds.primary.disabled,
                    bcursor::MoveToColumnAndRow(
                        (self.position().x + 1    ) as u16,
                        (self.position().y + 1 + i) as u16
                    ),
                    " ".repeat(self.size().x as usize)
                ).unwrap();
            }
        }

        // TODO: space conversion
        let (vx, vy) = self.cursor_visible_relative_position(editor, &file);

        // TODO: dont use real cursor, just draw them
        if vx < 1 || vy < 1 || vx > self.size().x || vy > self.size().y {
            write!(editor.stdout, "{}", bcursor::HIDE).unwrap();
        } else {
            write!(
                editor.stdout,
                "{}{}",
                bcursor::SHOW,
                bcursor::MoveToColumnAndRow(
                    (self.position().x + vx) as u16,
                    (self.position().y + vy) as u16
                )
            ).unwrap();
        }
    }

    fn handle_event(&mut self, editor: &mut Editor, event: Event) {
        match event {
            Event::Key(key) => match key {
                Key::Esc       => { self.exit           (editor); },
                Key::Ctrl('s') => { self.save           (editor); },
                Key::Left      => { self.left           (editor); },
                Key::Right     => { self.right          (editor); },
                Key::Up        => { self.up             (editor); },
                Key::Down      => { self.down           (editor); },
                Key::CtrlLeft  => { self.prev_word      (editor); },
                Key::CtrlRight => { self.next_word      (editor); },
                Key::Home      => { self.line_start     (editor); },
                Key::End       => { self.line_end       (editor); },
                Key::CtrlHome  => { self.file_start     (editor); },
                Key::CtrlEnd   => { self.file_end       (editor); },
                Key::Backspace => { self.erase_left     (editor); },
                Key::Delete    => { self.erase_right    (editor); },
                Key::Ctrl('h') => { self.erase_prev_word(editor); },
                Key::AltUp     => { self.move_line_up   (editor); },
                Key::AltDown   => { self.move_line_down (editor); },
                Key::Char(ch)  => match ch {
                    '\n'  => { self.newline  (editor       ); },
                    '\t'  => { self.tab      (editor       ); },
                    other => { self.character(editor, other); }
                },
                _ => ()
            },
            Event::Mouse(MouseEvent::Press(mouse_button, x, y)) => match mouse_button {
                MouseButton::Left      => { self.warp_cursor(editor,  x, y); },
                MouseButton::WheelUp   => { self.scroll     (editor, -1   ); },
                MouseButton::WheelDown => { self.scroll     (editor,  1   ); },
                _                      => ()
            },
            Event::Unsupported(bytes) => {
                if bytes == [27, 91, 51, 59, 53, 126] {
                    self.erase_next_word(editor);
                }
            },
            _ => ()
        }
    }
}
