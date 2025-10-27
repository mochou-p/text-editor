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
            Key::Up | Key::Down | Key::Left | Key::Right => self.handle_arrow_key(key),
            Key::Char(c)                                 => self.handle_char_key(c),
            Key::Backspace                               => self.handle_backspace(),
            Key::Esc                                     => { return true; },
            _                                            => ()
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
        let printable_option = match arrow_key {
            Key::Up    => Some(self.move_cursor_up()),
            Key::Down  => Some(self.move_cursor_down()),
            Key::Left  => Some(self.move_cursor_left()),
            Key::Right => Some(self.move_cursor_right()),
            _          => None
        };

        if let Some(Some(printable)) = printable_option {
            write!(self.stdout, "{printable}").unwrap();
            self.stdout.flush().unwrap();
        }
    }

    fn handle_char_key(&mut self, c: char) {
        self.insert_char(c);

        let printable = self.char_into_raw_print(c);
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

        self.lines[self.cursor.y].remove(self.cursor.x);

        write!(
            self.stdout,
            "{}{} {}",
            cursor::Left(1),
            &self.lines[self.cursor.y][self.cursor.x..],
            self.update_cursor_position()
        ).unwrap();
    }

    fn wrapping_backspace(&mut self) {
        let moved_line = self.lines.remove(self.cursor.y);

        self.cursor.y      -= 1;
        self.cursor.x       = self.lines[self.cursor.y].len();
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
                u16::try_from(word.start    + 1).unwrap(),
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
            "as" | "Err" | "false" | "None" | "Result" | "self" | "Some" | "true"
                => Some(&color::Cyan),
            "break" | "continue" | "crate" | "else" | "enum" | "extern" | "fn" | "for" | "if"
            | "impl" | "in" | "let" | "loop" | "match" | "mod" | "pub" | "return" | "struct"
            | "super" | "trait" | "type" | "use" | "where" | "while" | "async" | "await" | "dyn"
                => Some(&color::Blue),
            _
                => None
        }
    }

    // NOTE: doing this on every keystroke is quite redundant, but fine for now cause its simple
    fn parse_current_line_into_words(&self) -> Vec<Word> {
        let mut words      = Vec::with_capacity(128);
        let     line       = &self.lines[self.cursor.y];
        let mut line_start = 0;

        loop {
            let slice = &line[line_start..];

            let Some(start) = slice.find(|c: char| !c.is_whitespace()) else {
                // TODO: but why do i only get here once when spamming [Enter]?
                break;
            };

            let from_word_start = &slice[start..];
            let word_len_option = from_word_start.find(|c: char| c.is_whitespace());

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
    fn insert_char(&mut self, c: char) {
        if c == '\n' {
            self.lines.insert(self.cursor.y + 1, String::with_capacity(128));
        } else {
            self.lines[self.cursor.y].insert(self.cursor.x, c);
            self.cursor.x      += 1;
            self.cursor.last_x  = self.cursor.x;
        }
    }

    fn char_into_raw_print(&mut self, c: char) -> String {
        if c == '\n' {
            // TODO: make scrolling region end equal term height, not 200
            format!(
                "{}\x1b[{};200r{}\x1b[r",
                self.move_cursor_to_new_line(),
                self.cursor.y + 1,
                scroll::Down(1)
            )
        } else {
            format!(
                "{c}{}{}",
                &self.lines[self.cursor.y][self.cursor.x..],
                self.update_cursor_position()
            )
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
                .to_min_with(self.lines[self.cursor.y].len());
        }

        Some(self.update_cursor_position())
    }

    fn move_cursor_down(&mut self) -> Option<cursor::Goto> {
        let current_line_len = self.lines[self.cursor.y].len();

        if self.cursor.y == self.lines.len() - 1 {
            if self.cursor.x == current_line_len {
                self.cursor.last_x = current_line_len;
                return None;
            }

            self.cursor.x      = self.lines[self.cursor.y].len();
            self.cursor.last_x = self.cursor.x;
        } else {
            self.cursor.y += 1;

            self.cursor.x
                .to_max_with(self.cursor.last_x)
                .to_min_with(self.lines[self.cursor.y].len());
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
            self.cursor.x  = self.lines[self.cursor.y].len();
        } else {
            self.cursor.x -= 1;
        }

        self.cursor.last_x = self.cursor.x;

        Some(self.update_cursor_position())
    }

    fn move_cursor_right(&mut self) -> Option<cursor::Goto> {
        let current_line_len = self.lines[self.cursor.y].len();

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

    fn move_cursor_to_new_line(&mut self) -> cursor::Goto {
        self.cursor.y      += 1;
        self.cursor.x       = 0;
        self.cursor.last_x  = 0;

        self.update_cursor_position()
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

