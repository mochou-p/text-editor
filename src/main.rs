// mochou-p/text-editor/src/main.rs

mod config;
mod utils;
mod view;

use std::collections::HashMap;
use std::io::{self, Stdout, Write as _};
use std::panic::{set_hook, take_hook, catch_unwind, AssertUnwindSafe};
use std::sync::OnceLock;

use betterm::{clear, color, cursor, screen};

use termion::input::{MouseTerminal, TermRead as _};
use termion::raw::{RawTerminal, IntoRawMode as _};

use config::Theme;
use view::{View, Browsing, Editing, Files};


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

struct Editor {
    exit:     bool,
    stdout:   MouseTerminal<RawTerminal<Stdout>>,
    theme:    Theme,
    view:     String,
    views:    HashMap<String, Box<dyn View>>,
    files:    HashMap<String, File>
}

pub struct File {
    clean:   bool,
    cursors: Vec<Cursor>,
    lines:   Vec<String>
}

struct Cursor {
    last_x: isize,
    x:      isize,
    y:      isize
}

#[derive(Clone, Copy)]
struct Ivec2 {
    x: isize,
    y: isize
}

impl From<(u16, u16)> for Ivec2 {
    fn from(value: (u16, u16)) -> Self {
        Self { x: value.0 as isize, y: value.1 as isize }
    }
}

impl Default for Ivec2 {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Ivec2 {
    const ZERO: Self = Self { x: 0, y: 0 };
}

impl Editor {
    fn new() -> Self {
        Self {
            exit:   false,
            stdout: MouseTerminal::from(io::stdout().into_raw_mode().unwrap()),
            theme:  Theme::default(),
            view:   Editing::name(),
            views:  HashMap::new(),
            files:  HashMap::new()
        }
    }

    fn initialise(&mut self) {
        write!(
            self.stdout,
            "{}{}",
            screen::ENTER_ALTERNATE,
            clear::WHOLE_SCREEN
        ).unwrap();

        let editing = Editing::new(self);
        self.views.insert(Editing::name(), Box::new(editing));

        let browsing = Browsing::new(self);
        self.views.insert(Browsing::name(), Box::new(browsing));

        let files = Files::new(self);
        self.views.insert(Files::name(), Box::new(files));
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

        let _ = take_hook();
        self.shutdown();

        result.is_ok()
    }

    // TODO: clean up
    fn inner_run(&mut self) {
        let keys = self.views.keys().cloned().collect::<Vec<String>>();

        for key in &keys {
            let mut view = self.views.remove(key).unwrap();
            view.reprint(self);
            self.views.insert(key.to_owned(), view);
        }

        self.stdout.flush().unwrap();

        let stdin  = io::stdin();
        let handle = stdin.lock();

        for event in handle.events() {
            let event = event.unwrap();

            let     name = self.view.clone();
            let mut view = self.views.remove(&name).unwrap();
            view.handle_event(self, event);
            self.views.insert(name, view);

            for key in &keys {
                let mut view = self.views.remove(key).unwrap();
                view.reprint(self);
                self.views.insert(key.to_owned(), view);
            }

            if self.exit {
                break;
            }

            self.stdout.flush().unwrap();
        }
    }

    fn view<T: View + 'static>(&mut self) -> &mut T {
        (
            self.views
                .get_mut(&T::name())
                .unwrap()
                as &mut dyn std::any::Any
        )
            .downcast_mut::<T>()
            .unwrap()
    }
}
