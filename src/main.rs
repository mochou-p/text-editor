// mochou-p/text-editor/src/main.rs

mod actions;
mod config;
mod utils;

use std::collections::HashMap;
use std::io::{self, Stdout, Write as _};
use std::panic::{set_hook, take_hook, catch_unwind, AssertUnwindSafe};
use std::sync::OnceLock;

use betterm::{clear, color, cursor, screen};

use termion::event::{Event, Key, MouseEvent, MouseButton};
use termion::input::{MouseTerminal, TermRead as _};
use termion::raw::{RawTerminal, IntoRawMode as _};

use config::{Keybinds, Theme};
use utils::{ToWith, Utf8};


static PANIC_LOCATION: OnceLock<String> = OnceLock::new();
static PANIC_PAYLOAD:  OnceLock<String> = OnceLock::new();

fn main() {
    let was_ok = {
        Editor::new().run()
    };

    if !was_ok {
        eprintln!(
            "{}{} crashed! panic info:{}\n{}{}",
            color::FG_RED,
            env!("CARGO_CRATE_NAME"),
            color::UNSET_FG,
            PANIC_LOCATION.get().unwrap_or(&String::new()),
            PANIC_PAYLOAD.get().unwrap()
        );
    }
}

pub struct Editor {
    exit:     bool,
    stdout:   MouseTerminal<RawTerminal<Stdout>>,
    keybinds: Keybinds,
    theme:    Theme,
    files:    HashMap<String, File>,
    view:     View
}

struct File {
    clean:   bool,
    cursors: Vec<Cursor>,
    lines:   Vec<String>
}

struct View {
    file:     String,
    position: Ivec2,
    size:     Ivec2,
    scroll:   Ivec2
}

struct Cursor {
    last_x: isize,
    x:      isize,
    y:      isize
}

struct Ivec2 {
    x: isize,
    y: isize
}

impl Editor {
    fn new() -> Self {
        let     size   = termion::terminal_size().unwrap();
        let     file   = std::env::args().nth(1).unwrap();
        let     string = std::fs::read_to_string(&file).unwrap();
        let mut lines  = string.lines().map(str::to_owned).collect::<Vec<String>>();

        if lines.is_empty() || string.ends_with('\n') {
            lines.push(String::new());
        }

        Self {
            exit:     false,
            stdout:   MouseTerminal::from(io::stdout().into_raw_mode().unwrap()),
            keybinds: Keybinds::default(),
            theme:    Theme::default(),
            files:    HashMap::from([(file.clone(), File {
                clean:   true,
                cursors: vec![Cursor { last_x: 0, x: 0, y: 0 }],
                lines
            })]),
            view:     View {
                file,
                position: Ivec2 { x: 0,               y: 0               },
                size:     Ivec2 { x: size.0 as isize, y: size.1 as isize },
                scroll:   Ivec2 { x: 0,               y: 0               }
            }
        }
    }

    fn initialise(&mut self) {
        write!(
            self.stdout,
            "{}{}",
            screen::ENTER_ALTERNATE,
            clear::WHOLE_SCREEN
        ).unwrap();

        self.reprint();
    }

    fn shutdown(&mut self) {
        write!(
            self.stdout,
            "{}{}{}",
            cursor::SHOW,
            screen::LEAVE_ALTERNATE,
            betterm::RESET_ALL
        ).unwrap();

        self.stdout.flush().unwrap();
    }

    fn run(mut self) -> bool {
        set_hook(Box::new(|panic_info| {
            if let Some(location) = panic_info.location() {
                let _ = PANIC_LOCATION.set(
                    format!(
                        "{}:{}:{}:\n",
                        location.file(),
                        location.line(),
                        location.column()
                    )
                );
            }

            let payload = panic_info.payload();

            let _ = PANIC_PAYLOAD.set(
                if let Some(small_string) = payload.downcast_ref::<&str>() {
                    String::from(*small_string)
                } else if let Some(big_string) = payload.downcast_ref::<String>() {
                    big_string.clone()
                } else {
                    String::from("anonymous panic")
                }
            );
        }));

        let result = catch_unwind(AssertUnwindSafe(|| {
            self.initialise();
            self.inner_run();
        }));

        self.shutdown();
        let _ = take_hook();

        result.is_ok()
    }

    fn inner_run(&mut self) {
        let stdin  = io::stdin();
        let handle = stdin.lock();

        for event in handle.events() {
            let event = event.unwrap();

            if let Some(f) = self.keybinds.0.get(&event) {
                f(self);

                if self.exit {
                    break;
                }
            } else {
                match event {
                    Event::Key(Key::Char(ch)) => {
                        actions::typing::character(self, ch);
                    },
                    Event::Mouse(MouseEvent::Press(MouseButton::Left, x, y)) => {
                        self.warp_cursor(x, y);
                    },
                    Event::Mouse(MouseEvent::Press(MouseButton::WheelDown, _, _)) => {
                        actions::view::scroll_down(self);
                    },
                    Event::Mouse(MouseEvent::Press(MouseButton::WheelUp, _, _)) => {
                        actions::view::scroll_up(self);
                    },
                    _ => ()
                }
            }

            self.reprint();
        }
    }

    fn cursor_visible_relative_position(&self) -> (isize, isize) {
        let cursor = &self.files[&self.view.file].cursors[0];

        let x = cursor.x + 1 - self.view.scroll.x;
        let y = cursor.y + 1 - self.view.scroll.y;

        (x, y)
    }

    fn reprint(&mut self) {
        write!(
            self.stdout,
            "{}{}",
            color::UNSET_BG,
            clear::WHOLE_SCREEN
        ).unwrap();

        for i in 0..self.view.size.y {
            let x = self.view.scroll.x;
            let y = self.view.scroll.y + i;

            if y < self.files[&self.view.file].lines.len() as isize {
                let line         = &self.files[&self.view.file].lines[y as usize];
                let visible_line = line.utf8_range(x, x + self.view.size.x);
                let cursor_line  = y == self.files[&self.view.file].cursors[0].y;

                let style = if cursor_line {
                    (&self.theme.cursor_bg, &self.theme.cursor_fg)
                } else {
                    (&self.theme.bg,        &self.theme.fg       )
                };

                write!(
                    self.stdout,
                    "{}{}{}{visible_line}{}",
                    style.0,
                    style.1,
                    cursor::MoveToColumnAndRow(
                        (self.view.position.x + 1    ) as u16,
                        (self.view.position.y + 1 + i) as u16
                    ),
                    " ".repeat((self.view.size.x - visible_line.utf8_len()).max(0) as usize)
                ).unwrap();

                // TODO: this can be simplified a lot
                if !cursor_line && line.utf8_len() > self.view.scroll.x && line.ends_with(' ') {
                    let line_len = line.utf8_len() as usize;
                    let count    = line.rfind(|ch| ch != ' ')
                        .map(|n| line_len - n - 1)
                        .unwrap_or(line_len);

                    let start = (line_len - count) as isize;
                    if start < self.view.scroll.x + self.view.size.x {
                        let real_start    = self.view.position.x + 1 + start - self.view.scroll.x;
                        let left_overflow = (real_start - self.view.position.x - 1).min(0);

                        write!(
                            self.stdout,
                            "{}{}{}",
                            cursor::MoveToColumn((real_start - left_overflow) as u16),
                            self.theme.bad_trail,
                            " ".repeat(
                                (
                                    count.min((self.view.size.x - (start - self.view.scroll.x)) as usize) as isize
                                    -
                                    (-left_overflow)
                                ) as usize
                            )
                        ).unwrap();
                    }
                }

                if self.view.scroll.x > 0 {
                    write!(
                        self.stdout,
                        "{}{}<",
                        cursor::MoveToColumn((self.view.position.x + 1) as u16),
                        self.theme.overflow
                    ).unwrap();
                }

                if line.utf8_len() - self.view.scroll.x - self.view.size.x > 0 {
                    write!(
                        self.stdout,
                        "{}{}>",
                        cursor::MoveToColumn((self.view.position.x + self.view.size.x) as u16),
                        self.theme.overflow
                    ).unwrap();
                }
            } else {
                write!(
                    self.stdout,
                    "{}{}{}",
                    self.theme.void,
                    cursor::MoveToColumnAndRow(
                        (self.view.position.x + 1    ) as u16,
                        (self.view.position.y + 1 + i) as u16),
                    " ".repeat(self.view.size.x as usize)
                ).unwrap();
            }
        }

        let (vx, vy) = self.cursor_visible_relative_position();

        // TODO: with more cursors, just draw them if visible
        //       (actually, just draw all of them with invert (bg<=>fg))
        if vx < 1 || vy < 1 || vx > self.view.size.x || vy > self.view.size.y {
            write!(self.stdout, "{}", cursor::HIDE).unwrap();
        } else {
            write!(
                self.stdout,
                "{}{}",
                cursor::SHOW,
                cursor::MoveToColumnAndRow(
                    (self.view.position.x + vx) as u16,
                    (self.view.position.y + vy) as u16
                )
            ).unwrap();
        }

        self.theme.update();

        self.stdout.flush().unwrap();
    }

    fn snap_to_cursor(view: &mut View, cursor: &Cursor) {
        if cursor.y < view.scroll.y {
            view.scroll.y = cursor.y;
        } else if cursor.y > view.scroll.y + view.size.y - 1 {
            view.scroll.y = cursor.y - view.size.y + 1;
        }

        if cursor.x < view.scroll.x {
            view.scroll.x = cursor.x;
        } else if cursor.x > view.scroll.x + view.size.x - 1 {
            view.scroll.x = cursor.x - view.size.x + 1;
        }
    }

    fn warp_cursor(&mut self, x: u16, y: u16) {
        // TODO: when there are more views, only map from scroll space,
        //       position space will be fixed outside when events are passed

        let y = {
            let line_count = self.files[&self.view.file].lines.len() as isize;
            let file       = &mut self.files.get_mut(&self.view.file).unwrap();

            file.cursors.drain(1..);

            let cursor = &mut file.cursors[0];

            cursor.y = (y - 1) as isize - self.view.position.y + self.view.scroll.y;
            cursor.x = (x - 1) as isize - self.view.position.x + self.view.scroll.x;

            cursor.y.to_min_with(line_count - 1);
            cursor.y as usize
        };

        let line_len = self.files[&self.view.file].lines[y].utf8_len();
        let cursor   = &mut self.files.get_mut(&self.view.file).unwrap().cursors[0];

        cursor.x.to_min_with(line_len);
        cursor.last_x = (x - 1) as isize;
    }
}
