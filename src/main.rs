// text-editor/src/main.rs

mod to;
mod utf8;

use std::io::{self, Stdout, Write as _};

use betterm::{clear, color, cursor, screen, scroll, RESET_ALL};

use termion::event::{Event, Key, MouseEvent};
use termion::input::{MouseTerminal, TermRead as _};
use termion::raw::{RawTerminal, IntoRawMode as _};

use to::{ToMaxWith, ToMinWith};
use utf8::{Utf8Len, Utf8Remove, Utf8Range, Utf8Drain, Utf8SplitOff, Utf8Insert, Utf8Index};


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

struct Editor {
    temp_exit: bool,
    stdout:    SpecialStdout,
    file:      Option<String>,
    cursor:    Cursor,
    terminal:  Terminal,
    scroll:    Scroll,
    lines:     Vec<String>
}

// main functions
impl Editor {
    fn new() -> Self {
        let temp_exit = false;

        let stdout = MouseTerminal::from(
            io::stdout()
                .into_raw_mode()
                .unwrap()
        );

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

        Self { temp_exit, stdout, file, cursor, terminal, scroll, lines, }
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
            let should_exit = self.handle_event(event.unwrap());

            if should_exit {
                break;
            }

            if self.temp_exit {
                std::thread::sleep(std::time::Duration::from_secs(3));
                break;
            }
        }

        self.shutdown();
    }

    fn shutdown(&mut self) {
        write!(self.stdout, "{}", screen::LEAVE_ALTERNATE).unwrap();
        self.stdout.flush().unwrap();

        self.save_file();
    }
}

// filesystem
impl Editor {
    fn save_file(&mut self) {
        if let Some(file) = &self.file {
            if self.cursor.y != 0 && self.lines.last().unwrap().is_empty() {
                self.lines.push(String::new());
            }

            std::fs::write(file, self.lines.join("\n")).unwrap();
        }
    }
}

mod unsupported {
    pub const CTRL_DELETE: [u8; 6] = [27, 91, 51, 59, 53, 126];
}

// event handlers
impl Editor {
    fn handle_event(&mut self, event: Event) -> bool {
        // TODO: only this when its necessary
        write!(self.stdout, "{}", cursor::HIDE).unwrap();

        let should_exit = match event {
            Event::Key(key)           => self.handle_key_event(key),
            Event::Mouse(mouse_event) => Self::handle_mouse_event(mouse_event),
            Event::Unsupported(bytes) => {
                if bytes == unsupported::CTRL_DELETE {
                    self.handle_delete(true);
                }

                false
            }
        };

        // TODO: same here
        write!(self.stdout, "{}", cursor::SHOW).unwrap();
        self.stdout.flush().unwrap();

        should_exit
    }

    fn handle_key_event(&mut self, key: Key) -> bool {
        match key {
            Key::Up | Key::Down | Key::Left | Key::Right
                => self.handle_arrow_key(key),
            Key::CtrlLeft | Key::CtrlRight
                => self.handle_word_key(key),
            Key::Home | Key::End | Key::CtrlHome | Key::CtrlEnd
                => self.handle_edge_key(key),
            Key::Char(c)   => self.handle_char_key(c),
            Key::Backspace => self.handle_backspace(false),
            Key::Ctrl('h') => self.handle_backspace(true),
            Key::Delete    => self.handle_delete(false),
            Key::Ctrl('s') => self.save_file(),
            Key::Esc       => { return true; },
            _              => ()
        }

        false
    }

    const fn handle_mouse_event(_mouse_event: MouseEvent) -> bool {
        false
    }
}

// key event handlers
impl Editor {
    fn handle_arrow_key(&mut self, arrow_key: Key) {
        let printable_option2 = match arrow_key {
            Key::Up    => Some(self.move_cursor_up()),
            Key::Down  => Some(self.move_cursor_down()),
            Key::Left  => Some(self.move_cursor_left()),
            Key::Right => Some(self.move_cursor_right()),
            _          => None
        };

        if let Some(Some(printable)) = printable_option2 {
            write!(self.stdout, "{printable}").unwrap();
            self.stdout.flush().unwrap();
        }
    }

    fn handle_word_key(&mut self, word_key: Key) {
        let printable_option2 = match word_key {
            Key::CtrlLeft  => Some(self.move_cursor_to_prev_word()),
            Key::CtrlRight => Some(self.move_cursor_to_next_word()),
            _              => None
        };

        if let Some(Some(printable)) = printable_option2 {
            write!(self.stdout, "{printable}").unwrap();
            self.stdout.flush().unwrap();
        }
    }
    
    fn handle_edge_key(&mut self, edge_key: Key) {
        let printable_option2 = match edge_key {
            Key::Home     => Some(self.move_cursor_to_horizontal_start()),
            Key::End      => Some(self.move_cursor_to_horizontal_end()),
            Key::CtrlHome => Some(self.move_cursor_to_vertical_start()),
            Key::CtrlEnd  => Some(self.move_cursor_to_vertical_end()),
            _             => None
        };

        if let Some(Some(printable)) = printable_option2 {
            write!(self.stdout, "{printable}").unwrap();
            self.stdout.flush().unwrap();
        }
    }

    fn handle_char_key(&mut self, ch: char) {
        self.insert_char(ch);

        let printable = self.print_char(ch);
        write!(self.stdout, "{printable}").unwrap();
        self.stdout.flush().unwrap();

        self.refresh();
    }

    fn handle_backspace(&mut self, ctrl: bool) {
        if self.cursor.x == 0 {
            if self.cursor.y == 0 {
                self.cursor.last_x = 0;
                return;
            }

            self.wrapping_backspace();
        } else if ctrl {
            self.ctrl_backspace();
        } else {
            self.normal_backspace();
        }

        self.stdout.flush().unwrap();

        self.refresh();
    }

    fn handle_delete(&mut self, ctrl: bool) {
        let current_line_len = self.lines[self.cursor.y].utf8_len();

        if self.cursor.x == current_line_len {
            if self.cursor.y == self.lines.len() - 1 {
                self.cursor.last_x = current_line_len;
                return;
            }

            self.wrapping_delete();
        } else if ctrl {
            self.ctrl_delete();
        } else {
            self.normal_delete();
        }

        self.stdout.flush().unwrap();

        self.refresh();
    }
}

// backspace helpers
impl Editor {
    fn normal_backspace(&mut self) {
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

    fn ctrl_backspace(&mut self) {
        let start = self.utf8_position_of_left_whitespace();

        self.lines[self.cursor.y]
            .utf8_drain(start, self.cursor.x);

        self.cursor.x      = start;
        self.cursor.last_x = start;

        write!(
            self.stdout,
            "{}{}{}{}",
            self.update_cursor_position(),
            self.lines[self.cursor.y].utf8_range(self.cursor.x, self.lines[self.cursor.y].utf8_len()),
            clear::LINE_RIGHT_OF_CURSOR,
            self.update_cursor_position()
        ).unwrap();
    }

    fn wrapping_backspace(&mut self) {
        let moved_line = self.lines.remove(self.cursor.y);

        self.cursor.y      -= 1;
        self.cursor.x       = self.lines[self.cursor.y].utf8_len();
        self.cursor.last_x  = self.cursor.x;

        self.lines[self.cursor.y]
            .push_str(&moved_line);

        // TODO: make scrolling region end equal term height, not 200
        write!(
            self.stdout,
            "\x1b[{};200r{}\x1b[r{}{moved_line}",
            self.cursor.y + 1,
            scroll::UpBy(1),
            self.update_cursor_position()
        ).unwrap();
    }
}

// delete helpers
impl Editor {
    fn normal_delete(&mut self) {
        self.cursor.last_x = self.cursor.x;

        self.lines[self.cursor.y].utf8_remove(self.cursor.x);

        write!(
            self.stdout,
            "{} {}",
            &self.lines[self.cursor.y].utf8_range(self.cursor.x, self.lines[self.cursor.y].utf8_len()),
            self.update_cursor_position()
        ).unwrap();
    }

    fn ctrl_delete(&mut self) {
        let offset = self.utf8_offset_to_right_whitespace();

        self.lines[self.cursor.y]
            .utf8_drain(self.cursor.x, self.cursor.x + offset);

        self.cursor.last_x = self.cursor.x;

        write!(
            self.stdout,
            "{}{}{}",
            self.lines[self.cursor.y].utf8_range(self.cursor.x, self.lines[self.cursor.y].utf8_len()),
            clear::LINE_RIGHT_OF_CURSOR,
            self.update_cursor_position()
        ).unwrap();

        self.stdout.flush().unwrap();
    }

    fn wrapping_delete(&mut self) {
        let moved_line = self.lines.remove(self.cursor.y + 1);

        self.cursor.last_x = self.cursor.x;

        self.lines[self.cursor.y]
            .push_str(&moved_line);

        // TODO: make scrolling region end equal term height, not 200
        write!(
            self.stdout,
            "{moved_line}\x1b[{};200r{}\x1b[r{}",
            self.cursor.y + 2,
            scroll::UpBy(1),
            self.update_cursor_position()
        ).unwrap();
    }
}

struct Word {
    start: usize,
    end:   usize
}

impl Word {
    const fn from(line_start: usize, start: usize, end: usize) -> Self {
        Self {
            start: line_start + start,
            end:   line_start + end
        }
    }
}

// styling helpers
impl Editor {
    fn refresh(&mut self) {
        let words = self.parse_current_line_into_words();

        // self.refresh_indent_indicator(&words);
        self.refresh_colors(words);
    }

    // TODO: make it context-aware and more sophisticated
    fn refresh_colors(&mut self, words: Vec<Word>) {
        for word in words {
            self.try_colorise_word(word);
        }

        write!(
            self.stdout,
            "{}{}",
            self.update_cursor_position(),
            RESET_ALL
        ).unwrap();

        self.stdout.flush().unwrap();
    }

    fn try_colorise_word(&mut self, word: Word) {
        let line = &self.lines[self.cursor.y];
        let text = &line[word.start..word.end];

        let color = Self::get_text_color(text)
            .unwrap_or_else(|| RESET_ALL.to_string());

        write!(
            self.stdout,
            "{}{color}{text}",
            cursor::MoveToColumnAndRow(
                u16::try_from(
                    line[..word.start].utf8_len() + 1
                ).unwrap(),
                u16::try_from(self.cursor.y + 1 - self.scroll.y).unwrap()
            )
        ).unwrap();

        self.stdout.flush().unwrap();
    }

    fn get_text_color(text: &str) -> Option<String> {
        match text {
            "macro_rules!" | "unsafe"
                => Some(color::FG_RED.to_string()),
            "bool" | "char" | "const" | "f32" | "f64" | "i8" | "i16" | "i32" | "i64" | "i128"
            | "isize" | "move" | "mut" | "ref" | "Self" | "static" | "str" | "String" | "u8"
            | "u16" | "u32" | "u64" | "u128" | "usize"
                => Some(color::FG_YELLOW.to_string()),
            "as" | "Err" | "false" | "None" | "Ok" | "Option" | "Result" | "self" | "Some" | "true"
                => Some(color::FG_CYAN.to_string()),
            "break" | "continue" | "crate" | "else" | "enum" | "extern" | "fn" | "for" | "if"
            | "impl" | "in" | "let" | "loop" | "match" | "mod" | "pub" | "return" | "struct"
            | "super" | "trait" | "type" | "use" | "where" | "while" | "async" | "await" | "dyn"
                => Some(color::FG_BLUE.to_string()),
            _
                => None
        }
    }

    fn parse_current_line_into_words(&self) -> Vec<Word> {
        let mut words      = Vec::with_capacity(128);
        let     line       = &self.lines[self.cursor.y];
        let mut line_start = 0;

        loop {
            let slice = &line[line_start..];

            let Some(start) = slice.find(|ch: char| !ch.is_whitespace()) else {
                // NOTE: but why do i only get here once when spamming [Enter]?
                break;
            };

            let from_word_start = &slice[start..];
            let word_len_option = from_word_start.find(char::is_whitespace);

            if let Some(len) = word_len_option {
                let end  = start + len;
                let word = Word::from(line_start, start, end);

                words.push(word);
                line_start += end;
            } else {
                let end  = start + from_word_start.len();
                let word = Word::from(line_start, start, end);

                words.push(word);
                break;
            }
        }

        words
    }
}

// char helpers
impl Editor {
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
        match ch {
            '\n' => {
                // TODO: make scrolling region end equal term height, not 200
                format!(
                    "{}{}\x1b[{};200r{}\x1b[r",
                    clear::LINE_RIGHT_OF_CURSOR,
                    self.move_cursor_to_new_line(),
                    self.cursor.y + 1,
                    scroll::DownBy(1)
                )
            },
            '\t' => {
                self.cursor.x += self.chars_left_until_next_tab();

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
}

// cursor helpers
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

    fn move_cursor_to_horizontal_start(&mut self) -> Option<cursor::MoveToColumnAndRow> {
        self.cursor.last_x = 0;

        if self.cursor.x == 0 {
            return None;
        }

        self.cursor.x = 0;

        Some(self.update_cursor_position())
    }

    fn move_cursor_to_horizontal_end(&mut self) -> Option<cursor::MoveToColumnAndRow> {
        self.cursor.last_x = self.lines[self.cursor.y].utf8_len();

        if self.cursor.x == self.cursor.last_x {
            return None;
        }

        self.cursor.x = self.cursor.last_x;

        Some(self.update_cursor_position())
    }

    fn move_cursor_to_vertical_start(&mut self) -> Option<cursor::MoveToColumnAndRow> {
        if self.cursor.y == 0 {
            return None;
        }

        self.cursor.y = 0;

        self.cursor.x
            .to_max_with(self.cursor.last_x)
            .to_min_with(self.lines[self.cursor.y].utf8_len());

        Some(self.update_cursor_position())
    }

    fn move_cursor_to_vertical_end(&mut self) -> Option<cursor::MoveToColumnAndRow> {
        let last_line_i = self.lines.len() - 1;

        if self.cursor.y == last_line_i {
            return None;
        }

        self.cursor.y = last_line_i;

        self.cursor.x
            .to_max_with(self.cursor.last_x)
            .to_min_with(self.lines[self.cursor.y].utf8_len());

        Some(self.update_cursor_position())
    }

    fn move_cursor_up(&mut self) -> Option<cursor::MoveToColumnAndRow> {
        if self.cursor.y == 0 {
            if self.cursor.x == 0 {
                self.cursor.last_x = 0;
                return None;
            }

            self.cursor.x      = 0;
            self.cursor.last_x = self.cursor.x;
        } else {
            self.cursor.y -= 1;

            self.cursor.x
                .to_max_with(self.cursor.last_x)
                .to_min_with(self.lines[self.cursor.y].utf8_len());

            if self.cursor.y + 1 - self.scroll.y == 0 {
                self.scroll.y -= 1;

                write!(self.stdout, "{}", scroll::DOWN).unwrap();
                self.refresh();

                return None;
            }
        }

        Some(self.update_cursor_position())
    }

    fn move_cursor_down(&mut self) -> Option<cursor::MoveToColumnAndRow> {
        let current_line_len = self.lines[self.cursor.y].utf8_len();

        if self.cursor.y == self.lines.len() - 1 {
            if self.cursor.x == current_line_len {
                self.cursor.last_x = current_line_len;
                return None;
            }

            self.cursor.x      = self.lines[self.cursor.y].utf8_len();
            self.cursor.last_x = self.cursor.x;
        } else {
            self.cursor.y += 1;

            self.cursor.x
                .to_max_with(self.cursor.last_x)
                .to_min_with(self.lines[self.cursor.y].utf8_len());

            if self.cursor.y - 1 - self.scroll.y == self.terminal.height - 1 {
                self.scroll.y += 1;

                write!(self.stdout, "{}", scroll::UP).unwrap();
                self.refresh();

                return None;
            }
        }

        Some(self.update_cursor_position())
    }

    fn move_cursor_left(&mut self) -> Option<cursor::MoveToColumnAndRow> {
        if self.cursor.x == 0 {
            if self.cursor.y == 0 {
                self.cursor.last_x = 0;
                return None;
            }

            self.cursor.y -= 1;
            self.cursor.x  = self.lines[self.cursor.y].utf8_len();

            if self.cursor.y + 1 - self.scroll.y == 0 {
                self.scroll.y -= 1;

                write!(self.stdout, "{}", scroll::DOWN).unwrap();
                self.refresh();

                return None;
            }
        } else {
            self.cursor.x -= 1;
        }

        self.cursor.last_x = self.cursor.x;

        Some(self.update_cursor_position())
    }

    fn move_cursor_right(&mut self) -> Option<cursor::MoveToColumnAndRow> {
        let current_line_len = self.lines[self.cursor.y].utf8_len();

        if self.cursor.x == current_line_len {
            if self.cursor.y == self.lines.len() - 1 {
                self.cursor.last_x = current_line_len;
                return None;
            }

            self.cursor.y      += 1;
            self.cursor.x       = 0;
            self.cursor.last_x  = 0;

            if self.cursor.y - 1 - self.scroll.y == self.terminal.height - 1 {
                self.scroll.y += 1;

                write!(self.stdout, "{}", scroll::UP).unwrap();
                self.refresh();

                return None;
            }
        } else {
            self.cursor.x      += 1;
            self.cursor.last_x  = self.cursor.x;
        }

        Some(self.update_cursor_position())
    }

    fn move_cursor_to_prev_word(&mut self) -> Option<cursor::MoveToColumnAndRow> {
        if self.cursor.x == 0 {
            return self.move_cursor_left();
        }

        self.cursor.x      = self.utf8_position_of_left_whitespace();
        self.cursor.last_x = self.cursor.x;

        Some(self.update_cursor_position())
    }

    fn move_cursor_to_next_word(&mut self) -> Option<cursor::MoveToColumnAndRow> {
        if self.cursor.x == self.lines[self.cursor.y].utf8_len() {
            return self.move_cursor_right();
        }

        self.cursor.x      += self.utf8_offset_to_right_whitespace();
        self.cursor.last_x  = self.cursor.x;

        Some(self.update_cursor_position())
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

