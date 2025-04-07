// text-editor/src/editor.rs

use std::{
    convert::TryFrom,
    error::Error,
    io::{self, ErrorKind, stdout, Stdout},
    env, fs, panic
};

use crossterm::{
    event::{self, Event, KeyboardEnhancementFlags, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{self, ClearType},
    cursor, execute
};

use {
    super::{
        config::{HAlignment, Config},
        utils::CastResult
    },
    crate::error
};


#[derive(Default)]
pub struct LongestLine {
        index:  usize,
    pub length: u16
}

impl LongestLine {
    // FIXME: this temp approach scales with file size,
    //        so cache and sort later
    fn from(lines: &[String]) -> CastResult<Self, usize, u16> {
        let mut longest = Self::default();

        for (i, line) in lines.iter().enumerate() {
            let len = line.len();

            if len > longest.length as usize {
                longest.index  = i;
                longest.length = u16::try_from(len)?;
            }
        }

        Ok(longest)
    }
}

enum FileResult {
    Some((String, Vec<String>)),
    None,
    Err(String)
}

#[expect(clippy::module_name_repetitions)]
pub struct TextEditor {
    out:    Stdout,
    config: Config,
    file:   Option<String>,

    pub columns:   u16,
    pub rows:      u16,
        _cursor_x: u16,
    pub cursor_y:  u16,

    pub lines:        Vec<String>,
    pub longest_line: LongestLine
}

impl TextEditor {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let     out    = stdout();
        let     config = Config::load()?;
        let mut file   = None;

        let (columns, rows) = terminal::size()?;
        let     cursor_x = 0;
        let mut cursor_y = 0;

        let (lines, longest_line) = {
            match Self::try_load_file() {
                FileResult::Some((path, lines)) => {
                    let longest_line = LongestLine::from(&lines)?;

                    file     = Some(path);
                    cursor_y = u16::try_from(lines.len())? - 1;

                    (lines, longest_line)
                },
                FileResult::None => {
                    let mut lines = Vec::with_capacity(4096);
                    lines.push(String::with_capacity(256));
                    let longest_line = LongestLine::default();

                    (lines, longest_line)
                },
                FileResult::Err(string) => {
                    return Err(string.into());
                }
            }
        };

        Ok(Self {
            out, config, file,
            columns, rows, _cursor_x: cursor_x, cursor_y,
            lines, longest_line
        })
    }

    // file manipulation ///////////////////////////////////////////////////////////////

    fn try_load_file() -> FileResult {
        let args = env::args();

        match args.len() {
            0..=1 => FileResult::None,
            2 => {
                let path = args.last().unwrap();

                match fs::read_to_string(&path) {
                    Ok(file) => {
                        let lines = file
                            .lines()
                            .map(str::to_owned)
                            .collect();

                        FileResult::Some((path, lines))
                    },
                    Err(err) => {
                        match err.kind() {
                            ErrorKind::NotFound => {
                                error!("file `{}` does not exist", path);

                                FileResult::Err(err.to_string())
                            },
                            ErrorKind::PermissionDenied => {
                                error!("current user lacks read privilege to `{}`", path);

                                FileResult::Err(err.to_string())
                            },
                            _ => {
                                error!("std::fs::read_to_string failed to read `{}`", path);

                                FileResult::Err(err.to_string())
                            }
                        }
                    }
                }
            },
            _ => {
                error!("currently you can only open 1 file\n       correct usage: `cargo run` or `cargo run <FILE_PATH>`");

                FileResult::Err("invalid CLI arguments".into())
            }
        }
    }

    // terminal state //////////////////////////////////////////////////////////////////

    fn prepare_terminal(&mut self) -> Result<bool, Box<dyn Error>> {
        let (x, y) = (
            self.config.halignment.get_starting_x(self)?,
            self.config.valignment.get_y(self)?
        );

        execute!(
            self.out,
            terminal::EnterAlternateScreen,
            terminal::Clear(ClearType::FromCursorUp),
            event::PushKeyboardEnhancementFlags(
                KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                |
                KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                |
                KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
            )
        )?;

        let was_raw = {
            if terminal::is_raw_mode_enabled()? {
                true
            } else {
                terminal::enable_raw_mode()?;

                false
            }
        };

        if self.file.is_some() {
            self.reprint_previous_lines(false)?;
            self.cursor_y -= 1;
        }

        execute!(self.out, cursor::MoveTo(x, y))?;

        Ok(was_raw)
    }

    fn restore_terminal(was_raw: bool, out_option: Option<&mut Stdout>) -> io::Result<()> {
        if !was_raw {
            terminal::disable_raw_mode()?;
        }

        out_option.map_or_else(
            || execute!(
                io::stdout(),
                event::PopKeyboardEnhancementFlags,
                terminal::LeaveAlternateScreen
            ),
            |out| execute!(
                out,
                event::PopKeyboardEnhancementFlags,
                terminal::LeaveAlternateScreen
            )
        )
    }

    // main loop ///////////////////////////////////////////////////////////////////////

    pub fn run(mut self) -> Result<(), Box<dyn Error>> {
        let was_raw = self.prepare_terminal()?;

        panic::set_hook(Box::new(move |panic_info| {
            Self::restore_terminal(was_raw, None)
                .expect("failed to restore terminal after panic");

            eprintln!("{panic_info}");
        }));

        self.main_loop()?;

        let _ = panic::take_hook();

        Self::restore_terminal(was_raw, Some(&mut self.out))?;

        Ok(())
    }

    fn main_loop(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            match event::read()? {
                Event::Key(key_event) => {
                    if key_event.code == KeyCode::Esc {
                        break;
                    }

                    if key_event.kind == KeyEventKind::Release && matches!(key_event.code, KeyCode::Char(_) | KeyCode::Backspace | KeyCode::Enter) {
                        continue;
                    }

                    if !key_event.modifiers.is_empty() {
                        if !key_event.modifiers.contains(KeyModifiers::CONTROL) {
                            continue;
                        }

                        let KeyCode::Char(c) = key_event.code else { continue; };

                        if c != 's' {
                            continue;
                        }

                        todo!();

                        // continue;
                    }

                    match key_event.code {
                        KeyCode::Char(c) => {
                            self.push(c)?;
                        },
                        KeyCode::Backspace => {
                            self.pop()?;
                        },
                        KeyCode::Enter => {
                            self.newline()?;
                        },
                        _ => ()
                    }
                },
                Event::Resize(_columns, _lines) => {
                    todo!();
                },
                _ => ()
            }
        }

        Ok(())
    }

    // typing //////////////////////////////////////////////////////////////////////////

    fn push(&mut self, c: char) -> Result<(), Box<dyn Error>> {
        let y    = self.cursor_y as usize;
        let line = &mut self.lines[y];
        line.push(c);
        let len = line.len();

        if self.config.halignment.needs_longest_line() && len > self.longest_line.length as usize {
            self.longest_line.index  = y;
            self.longest_line.length = u16::try_from(len)?;

            execute!(self.out, terminal::Clear(ClearType::FromCursorUp))?;
            if self.cursor_y % 2 == 1 {
                self.cursor_y -= 1;
            }
            self.reprint_previous_lines(false)?;
            self.cursor_y -= 1;
        }

        self.reprint_current_line(false)?;

        Ok(())
    }

    fn pop(&mut self) -> Result<(), Box<dyn Error>> {
        let y = self.cursor_y as usize;

        if self.lines[y].is_empty() {
            // TODO
            return Ok(());
        }

        self.lines[y].pop();

        if self.config.halignment.needs_longest_line() && self.lines.len() > 1 && self.longest_line.index == y {
            self.longest_line.length -= 1;
            self.longest_line         = LongestLine::from(&self.lines)?;

            self.reprint_previous_lines(
                self.config.halignment == HAlignment::CenterLeft
            )?;

            self.cursor_y -= 1;
        } else {
            execute!(self.out, terminal::Clear(ClearType::CurrentLine))?;
        }

        self.reprint_current_line(false)?;

        Ok(())
    }

    fn newline(&mut self) -> Result<(), Box<dyn Error>> {
        {
            let len           = self.lines.len();
            let not_last_line = len != (self.cursor_y + 1) as usize;
            let overflow      = len == self.rows as usize;

            if not_last_line || overflow {
                todo!();
            }
        }

        self.reprint_previous_lines(false)?;
        self.lines.push(String::with_capacity(256));

        execute!(
            self.out,
            cursor::MoveDown(1),
            terminal::Clear(ClearType::CurrentLine)
        )?;

        self.reprint_current_line(false)?;

        Ok(())
    }

    // printing ////////////////////////////////////////////////////////////////////////

    fn reprint_current_line(&mut self, shrink: bool) -> Result<(), Box<dyn Error>> {
        let mut x = self.config.halignment.get_x(self)?;

        match self.config.halignment {
            HAlignment::Center | HAlignment::CenterRight | HAlignment::Right => {
                execute!(self.out, terminal::Clear(ClearType::CurrentLine))?;
            },
            _ => ()
        }

        if shrink {
            x -= 1;
        }

        let line = &self.lines[self.cursor_y as usize];
        execute!(self.out, cursor::MoveToColumn(x))?;

        if shrink {
            print!(" {line}");
        } else {
            print!("{line}");
        }

        execute!(self.out, terminal::Clear(ClearType::UntilNewLine))?;

        Ok(())
    }

    fn reprint_previous_lines(&mut self, shrink: bool) -> Result<(), Box<dyn Error>> {
        self.cursor_y = 0;

        for _ in 0..self.lines.len() {
            let y = self.config.valignment.get_y(self)?;

            execute!(self.out, cursor::MoveToRow(y))?;

            self.reprint_current_line(shrink)?;
            self.cursor_y += 1;
        }

        Ok(())
    }
}

