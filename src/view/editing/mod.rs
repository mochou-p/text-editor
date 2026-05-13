// mochou-p/text-editor/src/view/editing/mod.rs

mod actions;

use std::collections::HashMap;
use std::path::PathBuf;
use termion::event::{Event, Key, MouseButton, MouseEvent};
use super::{View, ViewData, Files};
use crate::{Editor, Cursor};
use crate::utils::{ToWith, Utf8};


pub struct Editing {
    view_data: ViewData,
    file:      Option<PathBuf>,
    files:     HashMap<PathBuf, File>
}

pub struct File {
    clean:   bool,
    cursors: Vec<Cursor>,
    lines:   Vec<String>
}

impl Editing {
    pub fn new() -> Self {
        Self {
            view_data: ViewData::fullscreen(),
            file:      None,
            files:     HashMap::new()
        }
    }

    fn read_file(path: &PathBuf) -> File {
        let     string = std::fs::read_to_string(path).unwrap();
        let mut lines  = string.lines().map(str::to_owned).collect::<Vec<String>>();

        if lines.is_empty() || string.ends_with('\n') {
            lines.push(String::new());
        }

        File {
            clean:   true,
            cursors: vec![Cursor { last_x: 0, x: 0, y: 0 }],
            lines
        }
    }

    pub fn open_file_from_browser(&mut self, editor: &mut Editor, path: PathBuf) {
        let file = Self::read_file(&path);
        self.files.insert(path.clone(), file);
        self.file = Some(path.clone());

        editor.view::<Files, ()>(|_, view| view.add_file(path.clone()));
    }

    pub fn open_file_from_files(&mut self, _editor: &mut Editor, path: PathBuf) {
        let file = Self::read_file(&path);
        self.files.insert(path.clone(), file);
        self.file = Some(path.clone());
    }

    fn cursor_visible_relative_position(&self, file: &PathBuf) -> (isize, isize) {
        let cursor = &self.files[file].cursors[0];

        let x = cursor.x + 1 - self.scroll().x;
        let y = cursor.y + 1 - self.scroll().y;

        (x, y)
    }

    fn snap_to_cursor(&mut self) {
        let cursor = {
            let Some(ref file) = self.file.as_ref().cloned() else { return; };
            self.files[file].cursors[0].clone()
        };

        if cursor.y < self.scroll().y {
            self.scroll_mut().y = cursor.y;
        } else if cursor.y > self.scroll().y + self.size().y - 1 {
            self.scroll_mut().y = cursor.y - self.size().y + 1;
        }

        if cursor.x < self.scroll().x {
            self.scroll_mut().x = cursor.x;
        } else if cursor.x > self.scroll().x + self.size().x - 1 {
            self.scroll_mut().x = cursor.x - self.size().x + 1;
        }
    }

    // TODO: wrong
    fn warp_cursor(&mut self, x: u16, y: u16) {
        let scroll         = self.scroll();
        let Some(ref file) = self.file.as_ref().cloned() else { return; };

        let y = {
            let line_count = self.files[file].lines.len() as isize;
            let file       = &mut self.files.get_mut(file).unwrap();

            file.cursors.drain(1..);

            let cursor = &mut file.cursors[0];

            cursor.y = y as isize + scroll.y;
            cursor.x = x as isize + scroll.x;

            cursor.y.to_min_with(line_count - 1);
            cursor.y as usize
        };

        let line_len = self.files[file].lines[y].utf8_len();
        let cursor   = &mut self.files.get_mut(file).unwrap().cursors[0];

        cursor.x.to_min_with(line_len);
        cursor.last_x = x as isize + scroll.x;
    }
}

impl View for Editing {
    fn any          (&mut self) -> &mut dyn std::any::Any { self                    }
    fn name         (         ) ->          String        { String::from("editing") }
    fn view_data    (&    self) -> &        ViewData      { &    self.view_data     }
    fn view_data_mut(&mut self) -> &mut     ViewData      { &mut self.view_data     }

    fn print_line(&mut self, editor: &mut Editor, buffer: &mut String, _loop_i: usize, scrolled_i: usize) {
        let Some(file) = self.file.as_ref().cloned() else {
            buffer.push_str(&format!(
                "{}{}",
                editor.theme.backgrounds.primary.disabled,
                " ".repeat(self.size().x as usize)
            ));

            return;
        };

        if scrolled_i < self.files[&file].lines.len() {
            let x = self.scroll().x;
            let y = scrolled_i as isize;

            let line         = &self.files[&file].lines[y as usize];
            let visible_line = line.utf8_range(x, x + self.size().x);
            let cursor_line  = y == self.files[&file].cursors[0].y;

            let style = if cursor_line {
                let (vx, vy)  = self.cursor_visible_relative_position(&file);
                editor.cursor = Some((self.position().x + vx, self.position().y + vy));

                (&editor.theme.backgrounds.primary.active, &editor.theme.foreground.active)
            } else {
                (&editor.theme.backgrounds.primary.normal, &editor.theme.foreground.normal)
            };

            buffer.push_str(&format!(
                "{}{}{visible_line}{}",
                style.0,
                style.1,
                " ".repeat((self.size().x - visible_line.utf8_len()).max(0) as usize)
            ));
        } else {
            buffer.push_str(&format!(
                "{}{}",
                editor.theme.backgrounds.primary.disabled,
                " ".repeat(self.size().x as usize)
            ));
        }

        /*
        for i in 0..self.size().y {
            if y < self.files[&file].lines.len() as isize {
                // TODO: this can be simplified a lot
                if !cursor_line && line.utf8_len() > self.scroll().x && line.ends_with(' ') {
                    let line_len = line.utf8_len() as usize;
                    let count    = line.rfind(|ch| ch != ' ')
                        .map(|n| line_len - n - 1)
                        .unwrap_or(line_len);

                    let start = (line_len - count) as isize;
                    if start < self.scroll().x + self.size().x {
                        let real_start    = self.position().x + 1 + start - self.scroll().x;
                        let left_overflow = (real_start - self.position().x - 1).min(0);

                        write!(
                            editor.stdout,
                            "{}{}{}",
                            bcursor::MoveToColumn((real_start - left_overflow) as u16),
                            editor.theme.special.error,
                            " ".repeat(
                                (
                                    count.min((self.size().x - (start - self.scroll().x)) as usize) as isize
                                    -
                                    (-left_overflow)
                                ) as usize
                            )
                        ).unwrap();
                    }
                }

                if self.scroll().x > 0 {
                    write!(
                        editor.stdout,
                        "{}{}<",
                        bcursor::MoveToColumn((self.position().x + 1) as u16),
                        editor.theme.special.overflow
                    ).unwrap();
                }

                if line.utf8_len() - self.scroll().x - self.size().x > 0 {
                    write!(
                        editor.stdout,
                        "{}{}>",
                        bcursor::MoveToColumn((self.position().x + self.size().x) as u16),
                        editor.theme.special.overflow
                    ).unwrap();
                }
            }
        }
        */
    }

    fn handle_event(&mut self, editor: &mut Editor, event: Event) {
        match event {
            Event::Key(key) => match key {
                Key::Esc       => { self.exit           (editor); },
                Key::Ctrl('s') => { self.save           (editor); },
                Key::Left      => { self.left           (      ); },
                Key::Right     => { self.right          (      ); },
                Key::Up        => { self.up             (      ); },
                Key::Down      => { self.down           (      ); },
                Key::CtrlLeft  => { self.prev_word      (      ); },
                Key::CtrlRight => { self.next_word      (      ); },
                Key::Home      => { self.line_start     (      ); },
                Key::End       => { self.line_end       (      ); },
                Key::CtrlHome  => { self.file_start     (      ); },
                Key::CtrlEnd   => { self.file_end       (      ); },
                Key::Backspace => { self.erase_left     (      ); },
                Key::Delete    => { self.erase_right    (      ); },
                Key::Ctrl('h') => { self.erase_prev_word(      ); },
                Key::AltUp     => { self.move_line_up   (      ); },
                Key::AltDown   => { self.move_line_down (      ); },
                Key::Char(ch)  => match ch {
                    '\n'  => { self.newline  (     ); },
                    '\t'  => { self.tab      (     ); },
                    other => { self.character(other); }
                },
                _ => ()
            },
            Event::Mouse(MouseEvent::Press(mouse_button, x, y)) => match mouse_button {
                MouseButton::Left      => { self.warp_cursor(x, y); },
                MouseButton::WheelUp   => { self.scroll_dir (-1  ); },
                MouseButton::WheelDown => { self.scroll_dir ( 1  ); },
                _                      => ()
            },
            Event::Unsupported(bytes) => {
                if bytes == [27, 91, 51, 59, 53, 126] {
                    self.erase_next_word();
                }
            },
            _ => ()
        }
    }
}
