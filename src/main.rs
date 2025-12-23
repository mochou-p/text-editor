// text-editor/src/main.rs

mod actions;
mod colors;
mod config;
mod events;
mod keybinds;
mod preferences;
mod to;
mod utf8;

use std::io::{self, Stdout, Write as _};

use betterm::{clear, cursor, screen, scroll};

use termion::event::{Event, Key};
use termion::input::{MouseTerminal, TermRead as _};
use termion::raw::{RawTerminal, IntoRawMode as _};

use actions::{Action, ACTIONS};
use config::Config;
use preferences::PreferenceMask;
use utf8::{Utf8Len, Utf8Remove, Utf8Range, Utf8SplitOff, Utf8Insert, Utf8Index};


fn main() {
    Editor::new().run();
}

type SpecialStdout = MouseTerminal<RawTerminal<Stdout>>;

#[derive(Default)]
struct Cursor {
    last_x: usize,
    x:      usize,
    y:      usize
}

struct Terminal {
    height: usize
}

#[derive(Default)]
struct Scroll {
    y: usize
}

pub struct Editor {
    exit:     bool,
    clean:    bool,
    stdout:   SpecialStdout,
    config:   Config,
    file:     Option<String>,
    cursor:   Cursor,
    terminal: Terminal,
    scroll:   Scroll,
    lines:    Vec<String>
}

// main functions
impl Editor {
    fn new() -> Self {
        let exit  = false;
        let clean = true;

        let stdout = MouseTerminal::from(
            io::stdout()
                .into_raw_mode()
                .unwrap()
        );

        let config = Config::default();

        let file = std::env::args().nth(1);

        let mut lines = {
            file.as_ref()
                .map_or_else(
                    || Vec::with_capacity(2048),
                    |file| {
                        std::fs::read_to_string(file)
                            .unwrap()
                            .lines()
                            .map(str::to_owned)
                            .collect()
                    }
                )
        };

        if lines.is_empty() {
            lines.push(String::with_capacity(128));
        }

        // TODO: save and restore last position
        let cursor = Cursor::default();

        let terminal = {
            let height = {
                let size = termion::terminal_size().unwrap();
                size.1 as usize
            };

            Terminal { height }
        };

        let scroll = Scroll::default();

        Self { exit, clean, stdout, config, file, cursor, terminal, scroll, lines }
    }

    fn initialise(&mut self) {
        write!(
            self.stdout,
            "{}{}",
            screen::ENTER_ALTERNATE,
            clear::WHOLE_SCREEN
        ).unwrap();

        if self.file.is_some() {
            for i in 0..self.lines.len().min(self.terminal.height) {
                self.cursor.y = i;
                self.refresh();
            }
        }

        // TODO: scroll to restored cursor
        self.cursor.y = 0;

        write!(
            self.stdout,
            "{}",
            self.update_cursor_position()
        ).unwrap();

        self.stdout.flush().unwrap();
    }

    fn run(mut self) {
        self.initialise();

        let stdin = io::stdin();

        for event in stdin.events() {
            let event = event.unwrap();

            // printable
            if let Event::Key(Key::Char(c)) = event {
                self.handle_char_key(c);
                continue;
            }

            if let Some(i) = self.config.get_bound_action_index(&event) {
                ACTIONS[*i](&mut self);

                if self.exit {
                    break;
                }
            }
        }

        self.shutdown();
    }

    fn shutdown(&mut self) {
        write!(self.stdout, "{}", screen::LEAVE_ALTERNATE).unwrap();
        self.stdout.flush().unwrap();

        if (self.config.preferences.0 & PreferenceMask::FILE_SAVE_ON_EDITOR_EXIT) != 0 {
            ACTIONS[Action::FILE_SAVE](self);
        }
    }
}

// internal not bindable actions
impl Editor {
    fn update_cursor_position(&self) -> cursor::MoveToColumnAndRow {
        let     x = self.cursor.x + 1;
        let mut y = self.cursor.y + 1;

        y -= self.scroll.y;

        cursor::MoveToColumnAndRow(
            u16::try_from(x).unwrap(),
            u16::try_from(y).unwrap()
        )
    }

    fn wrapping_move_left(&mut self) -> bool {
        if self.cursor.y == 0 {
            self.cursor.last_x = 0;
            return false;
        }

        self.cursor.y -= 1;
        self.cursor.x  = self.lines[self.cursor.y].utf8_len();

        if self.cursor.y + 1 - self.scroll.y == 0 {
            self.scroll.y -= 1;

            write!(self.stdout, "{}", scroll::DOWN).unwrap();
            self.refresh();

            // TODO: doesnt this leave last_x wrong?
            return false;
        }

        true
    }

    fn wrapping_move_right(&mut self) ->bool {
        let current_line_len = self.lines[self.cursor.y].utf8_len();

        if self.cursor.y == self.lines.len() - 1 {
            self.cursor.last_x = current_line_len;
            return false;
        }

        self.cursor.y      += 1;
        self.cursor.x       = 0;
        self.cursor.last_x  = 0;

        if self.cursor.y - 1 - self.scroll.y == self.terminal.height - 1 {
            self.scroll.y += 1;

            write!(self.stdout, "{}", scroll::UP).unwrap();
            self.refresh();

            return false;
        }

        true
    }

    fn handle_char_key(&mut self, ch: char) {
        self.clean = false;

        self.insert_char(ch);

        let printable = self.print_char(ch);
        write!(self.stdout, "{printable}").unwrap();
        self.stdout.flush().unwrap();

        self.refresh();
    }

    fn normal_erase_character_left(&mut self) {
        self.clean = false;

        self.cursor.x      -= 1;
        self.cursor.last_x  = self.cursor.x;

        self.lines[self.cursor.y].utf8_remove(self.cursor.x);

        write!(
            self.stdout,
            "{}{} {}",
            cursor::MOVE_LEFT,
            &self.lines[self.cursor.y].utf8_range(self.cursor.x, self.lines[self.cursor.y].utf8_len()),
            self.update_cursor_position()
        ).unwrap();
    }

    fn wrapping_erase_character_left(&mut self) {
        self.clean = false;

        let moved_line = self.lines.remove(self.cursor.y);

        self.cursor.y      -= 1;
        self.cursor.x       = self.lines[self.cursor.y].utf8_len();
        self.cursor.last_x  = self.cursor.x;

        self.lines[self.cursor.y]
            .push_str(&moved_line);

        let y = self.cursor.y - self.scroll.y;
        if y == self.terminal.height - 2 {
            self.cursor.y += 1;
            write!(self.stdout, "{}{}", cursor::MOVE_DOWN, clear::LINE_RIGHT_OF_CURSOR).unwrap();
            self.refresh();
            self.cursor.y -= 1;
        } else {
            write!(
                self.stdout,
                "\x1b[{};{}r{}\x1b[r",
                y + 2,
                self.terminal.height,
                scroll::UpBy(1)
            ).unwrap();

            let old_y     = self.cursor.y;
            self.cursor.y = self.terminal.height - y + self.cursor.y - 1;

            self.refresh();

            self.cursor.y = old_y;
        }
    }

    fn normal_erase_character_right(&mut self) {
        self.clean = false;

        self.cursor.last_x = self.cursor.x;

        self.lines[self.cursor.y].utf8_remove(self.cursor.x);

        write!(
            self.stdout,
            "{} {}",
            &self.lines[self.cursor.y].utf8_range(self.cursor.x, self.lines[self.cursor.y].utf8_len()),
            self.update_cursor_position()
        ).unwrap();
    }

    fn wrapping_erase_character_right(&mut self) {
        self.clean = false;

        let moved_line = self.lines.remove(self.cursor.y + 1);

        self.cursor.last_x = self.cursor.x;

        self.lines[self.cursor.y]
            .push_str(&moved_line);

        let y = self.cursor.y - self.scroll.y;
        if y == self.terminal.height - 1 {
        } else if y == self.terminal.height - 2 {
            self.cursor.y += 1;
            write!(self.stdout, "{}{}", cursor::MOVE_DOWN, clear::WHOLE_LINE).unwrap();
            self.refresh();
            self.cursor.y -= 1;
        } else {
            write!(
                self.stdout,
                "\x1b[{};{}r{}\x1b[r{}",
                y + 2,
                self.terminal.height,
                scroll::UpBy(1),
                self.update_cursor_position()
            ).unwrap();

            let old_y     = self.cursor.y;
            self.cursor.y = self.terminal.height - y + self.cursor.y - 1;

            self.refresh();

            self.cursor.y = old_y;
        }
    }

    fn insert_char(&mut self, ch: char) {
        match ch {
            '\n' => {
                let trail = self.lines[self.cursor.y].utf8_split_off(self.cursor.x);
                self.lines.insert(self.cursor.y + 1, trail);
            },
            '\t' => {
                let spaces = " ".repeat(self.chars_left_until_next_tab());

                self.lines[self.cursor.y]
                    .insert_str(
                        self.cursor.x,
                        &spaces
                    );
            },
            _ => {
                self.lines[self.cursor.y].utf8_insert(self.cursor.x, ch);
                self.cursor.x      += 1;
                self.cursor.last_x  = self.cursor.x;
            }
        }
    }

    fn print_char(&mut self, ch: char) -> String {
        self.clean = false;

        match ch {
            '\n' => {
                // TODO: temp?
                self.refresh();

                // TODO: temp
                let y = self.cursor.y - self.scroll.y;
                if y == self.terminal.height - 1 {
                    self.scroll.y += 1;

                    format!(
                        "{}{}{}",
                        clear::LINE_RIGHT_OF_CURSOR,
                        scroll::UP,
                        self.move_cursor_to_new_line()
                    )
                } else if y == self.terminal.height - 2 {
                    format!(
                        "{}{}{}",
                        clear::LINE_RIGHT_OF_CURSOR,
                        self.move_cursor_to_new_line(),
                        clear::LINE_RIGHT_OF_CURSOR
                    )
                } else {
                    format!(
                        "{}{}\x1b[{};{}r{}\x1b[r",
                        clear::LINE_RIGHT_OF_CURSOR,
                        self.move_cursor_to_new_line(),
                        y + 2,
                        self.terminal.height,
                        scroll::DOWN
                    )
                }
            },
            '\t' => {
                self.cursor.x += self.chars_left_until_next_tab();

                // TODO: temp
                write!(self.stdout, "{}", clear::WHOLE_LINE).unwrap();

                self.update_cursor_position().to_string()
            },
            _ => {
                format!(
                    "{ch}{}{}",
                    &self.lines[self.cursor.y].utf8_range(self.cursor.x, self.lines[self.cursor.y].utf8_len()),
                    self.update_cursor_position()
                )
            }
        }
    }

    fn move_cursor_to_new_line(&mut self) -> cursor::MoveToColumnAndRow {
        self.cursor.y      += 1;
        self.cursor.x       = 0;
        self.cursor.last_x  = 0;

        self.update_cursor_position()
    }

    const fn chars_left_until_next_tab(&self) -> usize {
        let n = 4 - (self.cursor.x % 4);

        if n == 0 { 4 } else { n }
    }
}

// utf-8 helpers (sorry if you have to read these lol)
impl Editor {
    fn utf8_position_of_left_whitespace(&self) -> usize {
        self.lines[self.cursor.y]
            .utf8_range(0, self.cursor.x)
            .rfind(|ch: char| !ch.is_whitespace())
            .map_or_else(
                || 0,
                |closest_word_to_left_absolute_end| {
                    self.lines[self.cursor.y]
                        .utf8_range(
                            0,
                            self.lines[self.cursor.y]
                                [..closest_word_to_left_absolute_end]
                                .utf8_len()
                        )
                        .rfind(char::is_whitespace)
                        .map_or_else(
                            || 0,
                            |whitespace_index| {
                                self.lines[self.cursor.y]
                                    [..whitespace_index]
                                    .utf8_len()
                                    + 1
                            }
                        )
                }
            )
    }

    fn utf8_offset_to_right_whitespace(&self) -> usize {
        let current_line_len = self.lines[self.cursor.y].utf8_len();

        self.lines[self.cursor.y]
            .utf8_range(
                self.cursor.x,
                self.lines[self.cursor.y].utf8_len()
            )
            .find(|ch: char| !ch.is_whitespace())
            .map_or_else(
                || current_line_len - self.cursor.x,
                |closest_word_to_right_relative_start| {
                    self.lines[self.cursor.y]
                        .utf8_range(
                            self.cursor.x + closest_word_to_right_relative_start,
                            self.lines[self.cursor.y].utf8_len()
                        )
                        .find(char::is_whitespace)
                        .map_or_else(
                            || current_line_len - self.cursor.x,
                            |distance_to_whitespace| {
                                let utf8_search_start = self.lines[self.cursor.y]
                                    .utf8_index(self.cursor.x + closest_word_to_right_relative_start);

                                let string_until_whitespace = &self.lines[self.cursor.y]
                                    [utf8_search_start..utf8_search_start + distance_to_whitespace];

                                let utf8_distance_to_whitespace = string_until_whitespace.utf8_len();

                                utf8_distance_to_whitespace + closest_word_to_right_relative_start
                            }
                        )
                }
            )
    }
}

