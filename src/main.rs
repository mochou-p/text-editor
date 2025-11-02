// text-editor/src/main.rs

use std::io::{self, Stdout, Write as _};

use termion::{clear, cursor, scroll};
use termion::color::{self, Color};
use termion::event::{Event, Key, MouseEvent};
use termion::input::{MouseTerminal, TermRead as _};
use termion::screen::{AlternateScreen, IntoAlternateScreen as _};
use termion::raw::{RawTerminal, IntoRawMode as _};


fn main() {
    let lines = {
        let mut editor = Editor::new();
        editor.initialise();
        editor.run();
        editor.lines
    };

    for (i, line) in lines.into_iter().enumerate() {
        println!(
            "{:>4}: {}^{}{line}{}${}",
            i + 1,
            color::Fg(color::Black),
            color::Fg(color::Reset),
            color::Fg(color::Black),
            color::Fg(color::Reset)
        );
    }
}

type SpecialStdout = MouseTerminal<AlternateScreen<RawTerminal<Stdout>>>;

#[derive(Default)]
struct Cursor {
    last_x: usize,
    x:      usize,
    y:      usize
}

struct Editor {
    stdout: SpecialStdout,
    cursor: Cursor,
    lines:  Vec<String>
}

// main functions
impl Editor {
    fn new() -> Self {
        let stdout = MouseTerminal::from(
            io::stdout()
                .into_raw_mode()
                .unwrap()
                .into_alternate_screen()
                .unwrap()
        );
        let cursor = Cursor::default();

        let mut lines = Vec::with_capacity(2048);
        lines.push(String::with_capacity(128));

        Self { stdout, cursor, lines }
    }

    fn initialise(&mut self) {
        write!(self.stdout, "{}{}", clear::All, self.update_cursor_position()).unwrap();
        self.stdout.flush().unwrap();
    }

    fn run(&mut self) {
        let stdin = io::stdin();

        for event in stdin.events() {
            let should_exit = self.handle_event(event.unwrap());

            if should_exit {
                break;
            }
        }
    }
}

// event handlers
impl Editor {
    fn handle_event(&mut self, event: Event) -> bool {
        match event {
            Event::Key(key)           => self.handle_key_event(key),
            Event::Mouse(mouse_event) => Self::handle_mouse_event(mouse_event),
            Event::Unsupported(_)     => false
        }
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
            Key::Backspace => self.handle_backspace(),
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

        self.try_refresh_colors();
    }

    fn handle_backspace(&mut self) {
        if self.cursor.x == 0 {
            if self.cursor.y == 0 {
                self.cursor.last_x = 0;
                return;
            }

            self.wrapping_backspace();
        } else {
            self.normal_backspace();
        }

        self.try_refresh_colors();
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
            cursor::Left(1),
            &self.lines[self.cursor.y].utf8_range(self.cursor.x, self.lines[self.cursor.y].utf8_len()),
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
            scroll::Up(1),
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

// coloring helpers
impl Editor {
    // TODO: make it context-aware and more sophisticated
    fn try_refresh_colors(&mut self) {
        let words = self.parse_current_line_into_words();

        for word in words {
            self.try_colorise_word(word);
        }

        write!(
            self.stdout,
            "{}{}",
            self.update_cursor_position(),
            color::Fg(color::Reset)
        ).unwrap();

        self.stdout.flush().unwrap();
    }

    fn try_colorise_word(&mut self, word: Word) {
        let line = &self.lines[self.cursor.y];
        let text = &line[word.start..word.end];

        let color = color::Fg(
            Self::get_text_color(text)
                .unwrap_or(&color::Reset)
        );

        write!(
            self.stdout,
            "{}{color}{text}",
            cursor::Goto(
                u16::try_from(
                    line[..word.start].utf8_len() + 1
                ).unwrap(),
                u16::try_from(self.cursor.y + 1).unwrap()
            )
        ).unwrap();
    }

    fn get_text_color(text: &str) -> Option<&dyn Color> {
        match text {
            "macro_rules!" | "unsafe"
                => Some(&color::Red),
            "bool" | "char" | "const" | "f32" | "f64" | "i8" | "i16" | "i32" | "i64" | "i128"
            | "isize" | "move" | "mut" | "ref" | "Self" | "static" | "str" | "String" | "u8"
            | "u16" | "u32" | "u64" | "u128" | "usize"
                => Some(&color::Yellow),
            "as" | "Err" | "false" | "None" | "Ok" | "Result" | "self" | "Some" | "true"
                => Some(&color::Cyan),
            "break" | "continue" | "crate" | "else" | "enum" | "extern" | "fn" | "for" | "if"
            | "impl" | "in" | "let" | "loop" | "match" | "mod" | "pub" | "return" | "struct"
            | "super" | "trait" | "type" | "use" | "where" | "while" | "async" | "await" | "dyn"
                => Some(&color::Blue),
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
                    clear::UntilNewline,
                    self.move_cursor_to_new_line(),
                    self.cursor.y + 1,
                    scroll::Down(1)
                )
            },
            '\t' => {
                self.cursor.x += self.chars_left_until_next_tab();

                self.update_cursor_position().into()
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
    fn update_cursor_position(&self) -> cursor::Goto {
        cursor::Goto(
            u16::try_from(self.cursor.x + 1).unwrap(),
            u16::try_from(self.cursor.y + 1).unwrap()
        )
    }

    fn move_cursor_to_horizontal_start(&mut self) -> Option<cursor::Goto> {
        self.cursor.last_x = 0;

        if self.cursor.x == 0 {
            return None;
        }

        self.cursor.x = 0;

        Some(self.update_cursor_position())
    }

    fn move_cursor_to_horizontal_end(&mut self) -> Option<cursor::Goto> {
        self.cursor.last_x = self.lines[self.cursor.y].utf8_len();

        if self.cursor.x == self.cursor.last_x {
            return None;
        }

        self.cursor.x = self.cursor.last_x;

        Some(self.update_cursor_position())
    }

    fn move_cursor_to_vertical_start(&mut self) -> Option<cursor::Goto> {
        if self.cursor.y == 0 {
            return None;
        }

        self.cursor.y = 0;

        self.cursor.x
            .to_max_with(self.cursor.last_x)
            .to_min_with(self.lines[self.cursor.y].utf8_len());

        Some(self.update_cursor_position())
    }

    fn move_cursor_to_vertical_end(&mut self) -> Option<cursor::Goto> {
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

    fn move_cursor_up(&mut self) -> Option<cursor::Goto> {
        if self.cursor.y == 0 {
            if self.cursor.x == 0 {
                self.cursor.last_x = 0;
                return None;
            }

            self.cursor.x = 0;
            self.cursor.last_x = self.cursor.x;
        } else {
            self.cursor.y -= 1;

            self.cursor.x
                .to_max_with(self.cursor.last_x)
                .to_min_with(self.lines[self.cursor.y].utf8_len());
        }

        Some(self.update_cursor_position())
    }

    fn move_cursor_down(&mut self) -> Option<cursor::Goto> {
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
        }

        Some(self.update_cursor_position())
    }

    fn move_cursor_left(&mut self) -> Option<cursor::Goto> {
        if self.cursor.x == 0 {
            if self.cursor.y == 0 {
                self.cursor.last_x = 0;
                return None;
            }

            self.cursor.y -= 1;
            self.cursor.x  = self.lines[self.cursor.y].utf8_len();
        } else {
            self.cursor.x -= 1;
        }

        self.cursor.last_x = self.cursor.x;

        Some(self.update_cursor_position())
    }

    fn move_cursor_right(&mut self) -> Option<cursor::Goto> {
        let current_line_len = self.lines[self.cursor.y].utf8_len();

        if self.cursor.x == current_line_len {
            if self.cursor.y == self.lines.len() - 1 {
                self.cursor.last_x = current_line_len;
                return None;
            }

            self.cursor.y      += 1;
            self.cursor.x       = 0;
            self.cursor.last_x  = 0;
        } else {
            self.cursor.x      += 1;
            self.cursor.last_x  = self.cursor.x;
        }

        Some(self.update_cursor_position())
    }

    fn move_cursor_to_prev_word(&mut self) -> Option<cursor::Goto> {
        if self.cursor.x == 0 {
            return self.move_cursor_left();
        }

        // TODO: clean up
        let rofl = {
            let closest_word_to_left_absolute_end_option = self.lines[self.cursor.y]
                .utf8_range(0, self.cursor.x)
                .rfind(|ch: char| !ch.is_whitespace());

            if let Some(closest_word_to_left_absolute_end) = closest_word_to_left_absolute_end_option {
                let idk = String::from(
                    &self.lines[self.cursor.y]
                        [..closest_word_to_left_absolute_end]
                ).utf8_len();

                let lol = self.lines[self.cursor.y]
                    .utf8_range(0, idk);

                if let Some(lmao) = lol.rfind(char::is_whitespace) {
                    String::from(
                        &self.lines[self.cursor.y]
                            [..lmao]
                    ).utf8_len() + 1
                } else {
                    0
                }
            } else {
                0
            }
        };

        self.cursor.x      = rofl;
        self.cursor.last_x = rofl;

        Some(self.update_cursor_position())
    }

    fn move_cursor_to_next_word(&mut self) -> Option<cursor::Goto> {
        let current_line_len = self.lines[self.cursor.y].utf8_len();

        if self.cursor.x == current_line_len {
            return self.move_cursor_right();
        }

        // TODO: clean up
        let utf8_distance_to_whitespace = {
            let closest_word_to_right_relative_start_option = self.lines[self.cursor.y]
                .utf8_range(
                    self.cursor.x,
                    self.lines[self.cursor.y].utf8_len()
                )
                .find(|ch: char| !ch.is_whitespace());

            if let Some(closest_word_to_right_relative_start) = closest_word_to_right_relative_start_option {
                let distance_to_whitespace_option = self.lines[self.cursor.y]
                    .utf8_range(
                        self.cursor.x + closest_word_to_right_relative_start,
                        self.lines[self.cursor.y].utf8_len()
                    )
                    .find(char::is_whitespace);

                if let Some(distance_to_whitespace) = distance_to_whitespace_option {
                    let utf8_search_start = self.lines[self.cursor.y]
                        .utf8_index(self.cursor.x + closest_word_to_right_relative_start);

                    let string_until_whitespace = &self.lines[self.cursor.y]
                        [utf8_search_start..utf8_search_start + distance_to_whitespace];

                    let utf8_distance_to_whitespace = string_until_whitespace.utf8_len();

                    utf8_distance_to_whitespace + closest_word_to_right_relative_start
                } else {
                    current_line_len - self.cursor.x
                }
            } else {
                current_line_len - self.cursor.x
            }
        };

        self.cursor.x      += utf8_distance_to_whitespace;
        self.cursor.last_x  = self.cursor.x;

        Some(self.update_cursor_position())
    }

    fn move_cursor_to_new_line(&mut self) -> cursor::Goto {
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

trait ToMinWith: Ord + Copy {
    fn to_min_with(&mut self, rhs: Self) -> &mut Self {
        *self = (*self).min(rhs);
        self
    }
}

impl ToMinWith for usize {}

trait ToMaxWith: Ord + Copy {
    fn to_max_with(&mut self, rhs: Self) -> &mut Self {
        *self = (*self).max(rhs);
        self
    }
}

impl ToMaxWith for usize {}

trait Utf8Len {
    fn utf8_len(&self) -> usize;
}

impl Utf8Len for str {
    fn utf8_len(&self) -> usize {
        self.chars().count()
    }
}

impl Utf8Len for String {
    fn utf8_len(&self) -> usize {
        self.as_str().utf8_len()
    }
}

trait Utf8Range {
    fn utf8_range(&self, start: usize, end: usize) -> String;
}

impl Utf8Range for str {
    fn utf8_range(&self, start: usize, end: usize) -> String {
        self.chars()
            .skip(start)
            .take(end - start)
            .collect()
    }
}

impl Utf8Range for String {
    fn utf8_range(&self, start: usize, end: usize) -> String {
        self.as_str().utf8_range(start, end)
    }
}

trait Utf8Index {
    fn utf8_index(&self, idx: usize) -> usize;
}

impl Utf8Index for str {
    fn utf8_index(&self, idx: usize) -> usize {
        self.char_indices()
            .nth(idx)
            .map_or(self.len(), |(i, _)| i)
    }
}

impl Utf8Index for String {
    fn utf8_index(&self, idx: usize) -> usize {
        self.as_str().utf8_index(idx)
    }
}

trait Utf8Insert {
    fn utf8_insert(&mut self, idx: usize, ch: char);
}

impl Utf8Insert for String {
    fn utf8_insert(&mut self, idx: usize, ch: char) {
        self.insert(self.utf8_index(idx), ch);
    }
}

trait Utf8Remove {
    fn utf8_remove(&mut self, idx: usize);
}

impl Utf8Remove for String {
    fn utf8_remove(&mut self, idx: usize) {
        self.remove(self.utf8_index(idx));
    }
}

trait Utf8SplitOff {
    fn utf8_split_off(&mut self, at: usize) -> String;
}

impl Utf8SplitOff for String {
    fn utf8_split_off(&mut self, at: usize) -> String {
        self.split_off(self.utf8_index(at))
    }
}

