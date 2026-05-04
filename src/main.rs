// mochou-p/text-editor/src/main.rs

mod config;
mod insert_set;
mod ivec2;
mod utils;
mod view;

use std::collections::HashMap;
use std::io::{self, Stdout, Write as _};
use std::panic::{set_hook, take_hook, catch_unwind, AssertUnwindSafe};
use std::sync::OnceLock;
use termion::event::{Event, MouseEvent, MouseButton};
use termion::input::{MouseTerminal, TermRead as _};
use termion::raw::{RawTerminal, IntoRawMode as _};
use betterm::{clear, color, cursor, screen};
use config::Theme;
use view::{View, Browsing, Editing, Files};

pub use {insert_set::InsertSet, ivec2::Ivec2};


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
    exit:   bool,
    stdout: MouseTerminal<RawTerminal<Stdout>>,
    theme:  Theme,
    cursor: Option<(isize, isize)>,
    view:   String,
    views:  HashMap<String, Box<dyn View>>
}

#[derive(Default, Clone)]
struct Cursor {
    last_x: isize,
    x:      isize,
    y:      isize
}

impl Editor {
    fn new() -> Self {
        Self {
            exit:   false,
            // NOTE: lock?
            stdout: MouseTerminal::from(io::stdout().into_raw_mode().unwrap()),
            theme:  Theme::default(),
            cursor: None,
            view:   Editing::name(),
            views:  HashMap::new()
        }
    }

    fn initialise(&mut self) {
        write!(
            self.stdout,
            "{}{}{}",
            cursor::HIDE,
            screen::ENTER_ALTERNATE,
            clear::WHOLE_SCREEN
        ).unwrap();

        let editing = Editing::new();
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
            screen::LEAVE_ALTERNATE,
            cursor::SHOW,
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

    fn try_update_focus(&mut self, event: &Event) {
        let Event::Mouse(MouseEvent::Press(MouseButton::Left, x, y)) = event else {
            return;
        };

        let x = *x as isize;
        let y = *y as isize;

        for name in self.views.keys() {
            let view = &self.views[name];

            let xy1 = view.position() + Ivec2::ONE;
            let xy2 = xy1 + view.size();

            if x >= xy1.x && y >= xy1.y && x <= xy2.x && y <= xy2.y {
                self.view = name.clone();
                return;
            }
        }
    }

    fn handle_event(&mut self, event: Event) {
        let     name = self.view.clone();
        let mut view = self.views.remove(&name).unwrap();

        view.handle_event(self, event);

        self.views.insert(name, view);
    }

    fn reprint_views(&mut self, keys: &[String], buffer: &mut String) {
        for key in keys {
            let mut view = self.views.remove(key).unwrap();

            for i in 0..view.size().y {
                buffer.clear();
                view.print_line(self, buffer, i as usize, (i + view.scroll().y) as usize);

                // TODO: cut printed width of String to size.x here somehow (+background fill)
                write!(
                    self.stdout,
                    "{}{buffer}",
                    cursor::MoveToColumnAndRow(
                        (view.position().x + 1)     as u16,
                        (view.position().y + 1 + i) as u16
                    )
                ).unwrap();
            }

            self.views.insert(String::from(key), view);
        }

        if let Some((x, y)) = self.cursor.take() {
            write!(
                self.stdout,
                "{}{}",
                cursor::SHOW,
                cursor::MoveToColumnAndRow(x as u16, y as u16)
            ).unwrap();
        } else {
            write!(self.stdout, "{}", cursor::HIDE).unwrap();
        }

        self.stdout.flush().unwrap();
    }

    fn inner_run(&mut self) {
        let     keys   = self.views.keys().cloned().collect::<Vec<String>>();
        let mut buffer = String::with_capacity(1024);

        self.reprint_views(&keys, &mut buffer);

        let stdin  = io::stdin();
        let handle = stdin.lock();

        for event in handle.events() {
            let event = event.unwrap();

            self.try_update_focus(&event);
            self.handle_event(event);

            if self.exit {
                break;
            }

            self.reprint_views(&keys, &mut buffer);
        }
    }

    fn view<T: View + 'static, R>(&mut self, f: impl Fn(&mut Self, &mut T) -> R) -> R {
        let name = T::name();

        let mut view = self.views
            .remove(&name)
            .unwrap();

        let t = view
            .any()
            .downcast_mut::<T>()
            .unwrap();

        let result = f(self, t);

        self.views.insert(name, view);

        result
    }
}
