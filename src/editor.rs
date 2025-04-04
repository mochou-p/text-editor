// text-editor/src/editor.rs

use std::{
    convert::TryFrom,
    error::Error,
    io::{self, stdout, Stdout}
};

use crossterm::{
    cursor,
    event::{self, Event, KeyboardEnhancementFlags, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{self, ClearType},
    execute
};

use super::config::{HAlignment, Config};


#[derive(Default)]
pub struct LongestLine {
        index:  usize,
    pub length: u16
}

#[expect(clippy::module_name_repetitions)]
pub struct TextEditor {
    out: Stdout,

    config: Config,

    pub columns:   u16,
    pub rows:      u16,
        _cursor_x: u16,
    pub cursor_y:  u16,

    pub lines:        Vec<String>,
    pub longest_line: LongestLine
}

impl TextEditor {
    pub fn new() -> io::Result<Self> {
        let out = stdout();

        let config = Config::load()?;

        let (columns, rows) = terminal::size()?;
        let cursor_x = 0;
        let cursor_y = 0;

        let mut lines = Vec::with_capacity(4096);
        lines.push(String::with_capacity(256));
        let longest_line = LongestLine::default();

        Ok(Self {
            out,
            config,
            columns, rows, _cursor_x: cursor_x, cursor_y,
            lines, longest_line
        })
    }

    // terminal state //////////////////////////////////////////////////////////////////

    fn prepare_terminal(&mut self) -> Result<(), Box<dyn Error>> {
        let (x, y) = (
            self.config.halignment.get_starting_x(self.columns),
            self.config.valignment.get_y(self)?
        );

        execute!(
            self.out,
            terminal::EnterAlternateScreen,
            terminal::Clear(ClearType::FromCursorUp),
            cursor::MoveTo(x, y),
            event::PushKeyboardEnhancementFlags(
                KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                |
                KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                |
                KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
            )
        )?;

        terminal::enable_raw_mode()?;

        Ok(())
    }

    fn restore_terminal(&mut self) -> io::Result<()> {
        terminal::disable_raw_mode()?;
        execute!(
            self.out,
            event::PopKeyboardEnhancementFlags,
            terminal::LeaveAlternateScreen
        )
    }

    // main loop ///////////////////////////////////////////////////////////////////////

    pub fn run(mut self) -> Result<(), Box<dyn Error>> {
        self.prepare_terminal()?;

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

        self.restore_terminal()?;

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
            self.longest_line         = self.find_longest_line()?;

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
        if self.lines.len() != (self.cursor_y + 1) as usize {
            todo!();
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

    // utils ///////////////////////////////////////////////////////////////////////////

    // FIXME: this temp approach scales with file size,
    //        so cache and sort later
    fn find_longest_line(&self) -> Result<LongestLine, <u16 as TryFrom<usize>>::Error> {
        let mut longest = LongestLine::default();

        for i in 0..self.lines.len() {
            let len = self.lines[i].len();

            if len > longest.length as usize {
                longest.index  = i;
                longest.length = u16::try_from(len)?;
            }
        }

        Ok(longest)
    }
}

