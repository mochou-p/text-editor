// text-editor/src/main.rs

use std::io::{self, Stdout, Write as _};

use termion::{clear, cursor};
use termion::event::{Event, Key, MouseEvent};
use termion::input::{MouseTerminal, TermRead as _};
use termion::screen::{AlternateScreen, IntoAlternateScreen as _};
use termion::raw::{RawTerminal, IntoRawMode as _};


fn main() {
    let mut editor = Editor::new();
    editor.initialise();
    editor.run();
}

type SpecialStdout = MouseTerminal<AlternateScreen<RawTerminal<Stdout>>>;

#[derive(Default)]
struct Cursor {
    x: u16,
    y: u16
}

struct Editor {
    stdout: SpecialStdout,
    cursor: Cursor
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

        Self {
            stdout,
            cursor: Cursor::default()
        }
    }

    fn initialise(&mut self) {
        write!(self.stdout, "{}{}", clear::All, self.update_cursor_position()).unwrap();
        self.stdout.flush().unwrap();
    }

    fn run(mut self) {
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
            Event::Mouse(mouse_event) => self.handle_mouse_event(mouse_event),
            Event::Unsupported(_)     => false
        }
    }

    fn handle_key_event(&mut self, key: Key) -> bool { 
        match key {
            Key::Char(c) => {
                let printable = self.char_into_raw_print(c);
                write!(self.stdout, "{printable}").unwrap();
                self.stdout.flush().unwrap();

                false
            },
            Key::Esc => true,
            _        => false
        }
    }

    fn handle_mouse_event(&mut self, _mouse_event: MouseEvent) -> bool {
        false
    }
}

// helpers
impl Editor {
    fn char_into_raw_print(&mut self, c: char) -> String {
        match c {
            '\n' => self.move_cursor_to_next_line().into(),
            c    => c.into()
        }
    }
}

// cursor helpers
impl Editor {
    fn update_cursor_position(&self) -> cursor::Goto {
        cursor::Goto(self.cursor.x + 1, self.cursor.y + 1)
    }

    fn move_cursor_to_next_line(&mut self) -> cursor::Goto {
        self.cursor.x  = 0;
        self.cursor.y += 1;

        self.update_cursor_position()
    }
}

