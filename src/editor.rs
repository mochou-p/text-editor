// text-editor/src/editor.rs

use std::{
    convert::TryFrom,
    error::Error,
    io::{self, stdout, Stdout}
};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    terminal::{self, ClearType},
    execute
};


#[expect(clippy::module_name_repetitions)]
pub struct TextEditor {
    out: Stdout,

    columns:  u16,
    rows:     u16,
    cursor_x: u16,
    cursor_y: u16,

    lines:        Vec<String>,
    longest_line: LongestLine
}

impl TextEditor {
    pub fn new() -> io::Result<Self> {
        let out = stdout();

        let (columns, rows) = terminal::size()?;
        let cursor_x = 0;
        let cursor_y = 0;

        let mut lines = Vec::with_capacity(4096);
        lines.push(String::with_capacity(256));
        let longest_line = LongestLine::default();

        Ok(Self {
            out,
            columns, rows, cursor_x, cursor_y,
            lines, longest_line
        })
    }

    // terminal state //////////////////////////////////////////////////////////////////

    fn prepare_terminal(&mut self) -> io::Result<()> {
        execute!(
            self.out,
            terminal::EnterAlternateScreen,
            cursor::MoveTo(
                self.cursor_x + self.columns / 2 - 1,
                self.cursor_y + self.rows    / 2 - 1
            )
        )?;

        terminal::enable_raw_mode()
    }

    fn restore_terminal(&mut self) -> io::Result<()> {
        terminal::disable_raw_mode()?;
        execute!(self.out, terminal::LeaveAlternateScreen)
    }

    // main loop ///////////////////////////////////////////////////////////////////////

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.prepare_terminal()?;

        loop {
            match event::read()? {
                Event::Key(key_event) => {
                    match key_event.code {
                        KeyCode::Esc => {
                            break;
                        },
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

        if len > self.longest_line.length as usize {
            self.longest_line.index  = y;
            self.longest_line.length = u16::try_from(len)?;

            execute!(self.out, terminal::Clear(ClearType::FromCursorUp))?;
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

        if self.lines.len() > 1 && self.longest_line.index == y {
            self.longest_line.length -= 1;
            self.longest_line         = self.find_longest_line()?;
            self.reprint_previous_lines(true)?;
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

    fn reprint_current_line(&mut self, shrink: bool) -> io::Result<()> {
        let mut x = self.centered_starting_column();

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

        execute!(self.out, terminal::Clear(ClearType::UntilNewLine))
    }

    fn reprint_previous_lines(&mut self, shrink: bool) -> Result<(), Box<dyn Error>> {
        self.cursor_y = 0;

        for _ in 0..self.lines.len() {
            let len = u16::try_from(self.lines.len())?;

            execute!(
                self.out,
                cursor::MoveToRow(
                    self.cursor_y
                    + self.rows
                        / 2
                    - len
                        / 2
                    - 1
                )
            )?;

            self.reprint_current_line(shrink)?;
            self.cursor_y += 1;
        }

        Ok(())
    }

    // utils ///////////////////////////////////////////////////////////////////////////

    const fn centered_starting_column(&self) -> u16 {
        self.columns
            / 2
        - 1
        - self.longest_line.length
            / 2
    }

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

#[derive(Default)]
struct LongestLine {
    index:  usize,
    length: u16
}

