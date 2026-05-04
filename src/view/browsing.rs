// mochou-p/text-editor/src/view/browsing.rs

use std::collections::HashMap;
use std::ffi::CString;
use std::path::PathBuf;
use termion::event::{Event, Key};
use super::{View, ViewData};
use super::editing::Editing;
use crate::Editor;


pub struct Browsing {
    view_data:   ViewData,
    current_dir: PathBuf,
    // TODO: cache i for each dir path
    focused:     usize,
    focuses:     HashMap<PathBuf, usize>,
    parent:      Option<BrowserEntry>,
    dirs:        Vec<BrowserEntry>,
    files:       Vec<BrowserEntry>
}

struct BrowserEntry {
    r:    bool,
    w:    bool,
    x:    bool,
    path: PathBuf
}

impl From<PathBuf> for BrowserEntry {
    fn from(value: PathBuf) -> Self {
        use std::os::unix::ffi::OsStrExt as _;

        let cstring = CString::new(value.as_os_str().as_bytes()).unwrap();
        let ptr     = cstring.as_ptr();

        let r = unsafe { libc::access(ptr, libc::R_OK) == 0 };
        let w = unsafe { libc::access(ptr, libc::W_OK) == 0 };
        let x = unsafe { libc::access(ptr, libc::X_OK) == 0 };

        Self { r, w, x, path: value }
    }
}

impl Browsing {
    pub fn new(editor: &mut Editor) -> Self {
        let view_data             = ViewData::left_of::<Editing>(editor, 24);
        let current_dir           = std::env::current_dir().unwrap();
        let (parent, dirs, files) = Self::load(&current_dir);
        let focused               = 0;
        let focuses               = HashMap::from([(current_dir.clone(), focused)]);

        Self { view_data, current_dir, focused, focuses, parent, dirs, files }
    }

    fn load(path: &PathBuf) -> (Option<BrowserEntry>, Vec<BrowserEntry>, Vec<BrowserEntry>) {
        let parent = if let Some(parent) = path.parent() {
            Some(BrowserEntry::from(parent.to_path_buf()))
        } else {
            None
        };

        let mut dirs  = Vec::new();
        let mut files = Vec::new();

        for entry in path.read_dir().unwrap() {
            let entry = entry.unwrap();

            if entry.metadata().unwrap().is_dir() {
                dirs.push(BrowserEntry::from(entry.path()));
            } else if entry.metadata().unwrap().is_file() {
                files.push(BrowserEntry::from(entry.path()));
            }
        }

        dirs .sort_by_key(|entry| entry.path.clone());
        files.sort_by_key(|entry| entry.path.clone());

        (parent, dirs, files)
    }

    fn up(&mut self) {
        if self.focused != 0 {
            self.focused -= 1;
        }
    }

    fn down(&mut self) {
        if self.focused != self.dirs.len() + self.files.len() - self.parent.is_none() as usize {
            self.focused += 1;
        }
    }

    fn go_out(&mut self) {
        let Some(parent) = self.parent.take() else { return; };

        let old_dir = self.current_dir.clone();
        self.focuses.insert(old_dir.clone(), self.focused);

        self.current_dir                     = parent.path;
        (self.parent, self.dirs, self.files) = Self::load(&self.current_dir);

        self.focused = self.focuses.get(&self.current_dir)
            .map_or_else(
                | | {
                    self.dirs
                        .iter()
                        .position(|entry| entry.path == old_dir)
                        .unwrap()
                    + self.parent.is_some() as usize
                },
                |i| *i
            );
    }

    fn go_in(&mut self, editor: &mut Editor) {
        let mut i = self.focused;

        if self.parent.is_some() {
            if i == 0 {
                self.go_out();
                return;
            }

            i -= 1;
        }

        if i < self.dirs.len() {
            self.focuses.insert(self.current_dir.clone(), self.focused);

            self.current_dir                     = self.dirs.remove(i).path;
            (self.parent, self.dirs, self.files) = Self::load(&self.current_dir);

            self.focused = self.focuses.get(&self.current_dir).map_or_else(|| 0, |i| *i);
        } else {
            i -= self.dirs.len() + self.parent.is_some() as usize - 1;
            editor.view::<Editing, ()>(|editor, view| view.open_file(editor, self.files[i].path.clone()));
        }
    }

    fn print_entry(
        &self,
        editor:    &Editor,
        buffer:    &mut String,
        focused:   bool,
        entry:     &BrowserEntry,
        is_parent: bool,
        prefix:    &str,
        suffix:    &str
    ) {
        buffer.push_str(
            if focused {
                &editor.theme.backgrounds.secondary.active
            } else {
                &editor.theme.backgrounds.secondary.normal
            }
        );

        let mut width = self.size().x;

        if width > 4 {
            buffer.push_str(&format!(
                "{}r{}w{}x ",
                if entry.r { &editor.theme.ansi.green } else { &editor.theme.ansi.red },
                if entry.w { &editor.theme.ansi.green } else { &editor.theme.ansi.red },
                if entry.x { &editor.theme.ansi.green } else { &editor.theme.ansi.red }
            ));

            width -= 4;
        }

        let path = if is_parent {
            String::from("..")
        } else {
            entry.path.file_name().unwrap().to_string_lossy().to_string()
        };

        let text         = format!("{path}{suffix}");
        let visible_text = &text[..(width as usize).min(text.len())];

        buffer.push_str(&format!(
            "{prefix}{visible_text}{}",
            " ".repeat(self.size().x as usize - visible_text.len() - 4)
        ));
    }

    fn print_dir(&self, editor: &Editor, buffer: &mut String, focused: bool, i: usize) {
        self.print_entry(editor, buffer, focused, &self.dirs[i], false, &editor.theme.ansi.blue, "/");
    }

    fn print_file(&self, editor: &Editor, buffer: &mut String, focused: bool, i: usize) {
        self.print_entry(editor, buffer, focused, &self.files[i], false, &editor.theme.foreground.normal, "");
    }

    fn print_empty(&mut self, editor: &Editor, buffer: &mut String) {
        buffer.push_str(&format!(
            "{}{}",
            &editor.theme.backgrounds.secondary.disabled,
            " ".repeat(self.size().x as usize)
        ));
    }
}

impl View for Browsing {
    fn any          (&mut self) -> &mut dyn std::any::Any { self                     }
    fn name         (         ) ->          String        { String::from("browsing") }
    fn view_data    (&    self) -> &        ViewData      { &    self.view_data      }
    fn view_data_mut(&mut self) -> &mut     ViewData      { &mut self.view_data      }

    fn print_line(&mut self, editor: &mut Editor, buffer: &mut String, _loop_i: usize, mut scrolled_i: usize) {
        if let Some(parent) = self.parent.as_ref() {
            if scrolled_i == 0 {
                self.print_entry(editor, buffer, self.focused == 0, parent, true, &editor.theme.ansi.blue, "/");
                return;
            } else {
                scrolled_i -= 1;
            }
        }

        let focused = self.focused == scrolled_i + self.parent.is_some() as usize;

        if scrolled_i < self.dirs.len() {
            self.print_dir(editor, buffer, focused, scrolled_i);
            return;
        }
        scrolled_i -= self.dirs.len();

        if scrolled_i < self.files.len() {
            self.print_file(editor, buffer, focused, scrolled_i);
            return;
        }

        self.print_empty(editor, buffer);
    }

    fn handle_event(&mut self, editor: &mut Editor, event: Event) {
        match event {
            Event::Key(key) => match key {
                Key::Up                      => { self.up()          },
                Key::Down                    => { self.down()        },
                Key::Left  | Key::Backspace  => { self.go_out()      },
                Key::Right | Key::Char('\n') => { self.go_in(editor) },
                _                            => ()
            },
            _ => ()
        }
    }
}
