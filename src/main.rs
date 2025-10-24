// text-editor/src/main.rs

use std::io::{self, Stdout, Write as _};

use termion::{clear, color, cursor};
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
            Key::Esc                                     => true,
            _                                            => false
        }
    }

    fn handle_mouse_event(_mouse_event: MouseEvent) -> bool {
        false
    }
}

// key event handlers
impl Editor {
    fn handle_arrow_key(&mut self, arrow_key: Key) -> bool {
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

        false
    }

    fn handle_char_key(&mut self, c: char) -> bool {
        self.insert_char(c);

        let printable = self.char_into_raw_print(c);
        write!(self.stdout, "{printable}").unwrap();
        self.stdout.flush().unwrap();

        false
    }
}

// char helpers
impl Editor {
    fn insert_char(&mut self, c: char) {
        if c == '\n' {
            self.lines.push(String::with_capacity(128));
        } else {
            self.lines[self.cursor.y].insert(self.cursor.x, c);
            self.cursor.x += 1;
        }
    }

    fn char_into_raw_print(&mut self, c: char) -> String {
        if c == '\n' {
            self.move_cursor_to_next_line().into()
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
                return None;
            }

            self.cursor.x = 0;
        } else {
            self.cursor.y -= 1;
            self.cursor.x.to_min_with(self.lines[self.cursor.y].len());
        }

        Some(self.update_cursor_position())
    }

    fn move_cursor_down(&mut self) -> Option<cursor::Goto> {
        if self.cursor.y == self.lines.len() - 1 {
            if self.cursor.x == self.lines[self.cursor.y].len() {
                return None;
            }

            self.cursor.x = self.lines[self.cursor.y].len();
        } else {
            self.cursor.y += 1;
            self.cursor.x.to_min_with(self.lines[self.cursor.y].len());
        }

        Some(self.update_cursor_position())
    }

    fn move_cursor_left(&mut self) -> Option<cursor::Goto> {
        if self.cursor.x == 0 {
            if self.cursor.y == 0 {
                return None;
            }

            self.cursor.y -= 1;
            self.cursor.x  = self.lines[self.cursor.y].len();
        } else {
            self.cursor.x -= 1;
        }

        Some(self.update_cursor_position())
    }

    fn move_cursor_right(&mut self) -> Option<cursor::Goto> {
        if self.cursor.x == self.lines[self.cursor.y].len() {
            if self.cursor.y == self.lines.len() - 1 {
                return None;
            }

            self.cursor.y += 1;
            self.cursor.x  = 0;
        } else {
            self.cursor.x += 1;
        }

        Some(self.update_cursor_position())
    }

    // TODO: refactor this to get rid of the note
    // NOTE: beware when using this from another helper,
    //       it will always move down, so dont use it
    //       from a call that can be bound by self.lines.len()
    fn move_cursor_to_next_line(&mut self) -> cursor::Goto {
        self.cursor.x  = 0;
        self.cursor.y += 1;

        self.update_cursor_position()
    }
}

trait ToMinWith: Ord + Copy {
    fn to_min_with(&mut self, rhs: Self) {
        *self = (*self).min(rhs);
    }
}

impl ToMinWith for usize {}

