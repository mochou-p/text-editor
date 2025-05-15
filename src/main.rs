// text-editor/src/main.rs

use libc::{
    fcntl, ioctl, iscntrl, tcgetattr, tcsetattr, winsize,
    BRKINT, CS8, ECHO, F_GETFL, F_SETFL, ICANON, ICRNL, IEXTEN, INPCK, ISIG, ISTRIP, IXON, O_NONBLOCK, OPOST, STDIN_FILENO, STDOUT_FILENO, TCSAFLUSH, TIOCGWINSZ
};

use std::{
    io::{stdin, stdout, Error, Read as _, Write as _},
    mem::zeroed
};


static CSI: &str = "\x1b[";

static FG: &str = "38;2;";
static BG: &str = "48;2;";

static RESET: &str = "0m";
static RED:   &str = "31m";

// Catppuccin Mocha (https://github.com/catppuccin/catppuccin)
static MANTLE:    &str = "24;24;37m";
static BASE:      &str = "30;30;46m";
static SURFACE_0: &str = "49;50;68m";
#[cfg(debug_assertions)]
static SURFACE_1: &str = "69;71;90m";
#[cfg(debug_assertions)]
static OVERLAY_0: &str = "108;112;134m";
static SUBTEXT_0: &str = "166;173;200m";
static TEXT:      &str = "205;214;244m";

fn main() {
    Editor::default().and_or(
        Editor::run,
        |error| eprintln!("{CSI}{RED}error{CSI}{RESET}: {error}")
    );
}

enum EditorResult {
    Ok(Editor),
    Err(String)
}

impl EditorResult {
    fn and_or(self, ok_f: impl FnOnce(Editor), err_f: impl FnOnce(String)) {
        match self {
            Self::Ok(editor) =>  ok_f(editor),
            Self::Err(error) => err_f(error)
        }
    }
}

struct Cursor {
    last_x: usize,
    x:      usize,
    y:      usize
}

struct Editor {
    cursor: Cursor,
    size:   winsize,
    lines:  Vec<String>
}

impl Editor {
    fn default() -> EditorResult {
        let mut size: winsize = unsafe { zeroed() };

        if unsafe { ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut size) } != 0 {
            return EditorResult::Err(
                format!(
                    "`libc::ioctl` returned \"{}\"",
                    Error::last_os_error()
                )
            );
        }

        let cursor    = Cursor { last_x: 0, x: 0, y: 0 };
        let mut lines = Vec::with_capacity(2048);
        lines.push(String::with_capacity(256));

        EditorResult::Ok(Self { cursor, size, lines })
    }

    #[expect(clippy::too_many_lines, clippy::cognitive_complexity, reason = "temp")]
    fn run(mut self) {
        let mut original_termios = unsafe { zeroed() };

        if unsafe { tcgetattr(STDIN_FILENO, &raw mut original_termios) } != 0 {
            eprintln!(
                "{CSI}{RED}error{CSI}{RESET}: `libc::tcgetattr` returned \"{}\"",
                Error::last_os_error()
            );
            return;
        }

        let mut new_termios  = original_termios;
        new_termios.c_iflag &= !(BRKINT | ICRNL | INPCK | ISTRIP | IXON);
        new_termios.c_oflag &= !(OPOST);
        new_termios.c_cflag |=   CS8;
        new_termios.c_lflag &= !(ECHO | ICANON | IEXTEN | ISIG);

        let mut stdin  = stdin();
        let mut stdout = stdout();
        let mut buffer = [0u8; 1];
        let mut idklol = [0u8; 2];

        if unsafe { tcsetattr(STDIN_FILENO, TCSAFLUSH, &raw const new_termios) } != 0 {
            eprintln!(
                "{CSI}{RED}error{CSI}{RESET}: `libc::tcsetattr` returned \"{}\"",
                Error::last_os_error()
            );
            return;
        }

        hide();
        alternative_screen();
        clear();

        move_to(1, 1);
        self.header();
        self.text_area();

        move_to(1, 2);
        show();

        loop {
            let _ = stdout.flush();

            match stdin.read_exact(&mut buffer) {
                Ok(()) => {
                    if unsafe { iscntrl(i32::from(buffer[0])) } == 0 {
                        let c = char::from(buffer[0]);
                        print!("{c}");
                        self.lines[self.cursor.y].insert(self.cursor.x, c);
                        self.cursor.x += 1;

                        if self.cursor.x < self.lines[self.cursor.y].len() {
                            print!("{}", &self.lines[self.cursor.y][self.cursor.x..]);
                            move_to_x(self.cursor.x + 1);
                        }

                        self.cursor.last_x = self.cursor.x;
                    } else {
                        match buffer[0] {
                            13 => {  // Enter
                                self.cursor.y += 1;
                                self.lines.insert(self.cursor.y, String::with_capacity(256));

                                if self.cursor.x < self.lines[self.cursor.y - 1].len() {
                                    print!("{CSI}0K");
                                    let remainder = self.lines[self.cursor.y - 1].drain(self.cursor.x..).collect::<String>();
                                    self.lines[self.cursor.y].push_str(&remainder);
                                }

                                move_to_next_line(1);
                                self.cursor.x = 0;
                                print!("{}", self.lines[self.cursor.y..].join(&format!("{CSI}0K{CSI}1E")));
                                self.line_background(true);
                                self.update_cursor();

                                self.cursor.last_x = self.cursor.x;
                            },
                            27 => {
                                if !Self::try_read_special(&mut idklol) {  // Escape
                                    break;
                                }

                                match idklol[1] {
                                    65 => {  // ArrowUp/ScrollUp
                                        if self.cursor.y == 0 {
                                            continue;
                                        }

                                        self.cursor.y -= 1;
                                        let len = self.lines[self.cursor.y].len();
                                        if self.cursor.last_x > self.cursor.x {
                                            self.cursor.x = self.cursor.last_x.min(len);
                                            self.update_cursor();
                                        } else if self.cursor.x > len {
                                            self.cursor.last_x = self.cursor.x;
                                            self.cursor.x      = len;
                                            self.update_cursor();
                                        } else {
                                            print!("{CSI}1A");
                                        }
                                    },
                                    66 => {  // ArrowDown/ScrollDown
                                        if self.cursor.y == self.lines.len() - 1 {
                                            continue;
                                        }

                                        self.cursor.y += 1;
                                        let len = self.lines[self.cursor.y].len();
                                        if self.cursor.last_x > self.cursor.x {
                                            self.cursor.x = self.cursor.last_x.min(len);
                                            self.update_cursor();
                                        } else if self.cursor.x > len {
                                            self.cursor.last_x = self.cursor.x;
                                            self.cursor.x      = len;
                                            self.update_cursor();
                                        } else {
                                            print!("{CSI}1B");
                                        }
                                    },
                                    67 => {  // ArrowRight
                                        if self.cursor.x == self.lines[self.cursor.y].len() {
                                            if self.cursor.y < self.lines.len() - 1 {
                                                move_to_next_line(1);
                                                self.cursor.x  = 0;
                                                self.cursor.y += 1;
                                            }
                                        } else {
                                            print!("{CSI}1C");
                                            self.cursor.x += 1;
                                        }

                                        self.cursor.last_x = self.cursor.x;
                                    },
                                    68 => {  // ArrowLeft
                                        if self.cursor.x == 0 {
                                            if self.cursor.y != 0 {
                                                self.cursor.y -= 1;
                                                self.cursor.x  = self.lines[self.cursor.y].len();
                                                self.update_cursor();
                                            }
                                        } else {
                                            print!("{CSI}1D");
                                            self.cursor.x -= 1;
                                        }

                                        self.cursor.last_x = self.cursor.x;
                                    },
                                    _ => ()
                                }
                            },
                            127 => {  // Backspace
                                if self.cursor.x == 0 {
                                    if self.cursor.y == 0 {
                                        self.cursor.last_x = self.cursor.x;
                                        continue;
                                    }

                                    if self.lines[self.cursor.y].is_empty() {
                                        self.lines.remove(self.cursor.y);

                                        print!("{}", self.lines[self.cursor.y..].join(&format!("{CSI}0K{CSI}1E")));
                                        if self.cursor.y != self.lines.len() {
                                            print!("{CSI}0K{CSI}1E");
                                        }

                                        self.line_background(false);
                                        self.cursor.y -= 1;
                                        self.cursor.x  = self.lines[self.cursor.y].len();
                                        self.update_cursor();
                                    } else {
                                        let len  = self.lines[self.cursor.y - 1].len();
                                        let line = self.lines.remove(self.cursor.y);
                                        self.lines[self.cursor.y - 1].push_str(&line);

                                        print!("{}", self.lines[self.cursor.y..].join(&format!("{CSI}0K{CSI}1E")));
                                        if self.cursor.y != self.lines.len() {
                                            print!("{CSI}0K{CSI}1E");
                                        }

                                        self.line_background(false);
                                        self.cursor.y -= 1;
                                        move_to(len + 1, self.cursor.y + 2);
                                        print!("{line}");
                                        move_to(len + 1, self.cursor.y + 2);
                                        self.cursor.x = len;
                                    }
                                } else if self.cursor.x == self.lines[self.cursor.y].len() {
                                    move_left(1);
                                    print!(" ");
                                    move_left(1);
                                    self.lines[self.cursor.y].pop();
                                    self.cursor.x -= 1;
                                } else {
                                    move_left(1);
                                    print!("{} ", &self.lines[self.cursor.y][self.cursor.x..]);
                                    move_to_x(self.cursor.x);
                                    self.cursor.x -= 1;
                                    self.lines[self.cursor.y].remove(self.cursor.x);
                                }

                                self.cursor.last_x = self.cursor.x;
                            },
                            _ => {
                                #[cfg(debug_assertions)]
                                print!("{CSI}{FG}{SURFACE_1}[{}]{CSI}{RESET}", buffer[0]);
                            }
                        }
                    }
                },
                Err(err) => {
                    normal_screen();
                    unsafe { tcsetattr(STDIN_FILENO, TCSAFLUSH, &raw const original_termios) };
                    eprintln!("{CSI}{RED}error{CSI}{RESET}: `Stdin::read_exact returned \"{err}\"");
                    return;
                }
            }
        }

        print!("{CSI}{RESET}");
        normal_screen();
        unsafe { tcsetattr(STDIN_FILENO, TCSAFLUSH, &raw const original_termios) };

        #[cfg(debug_assertions)]
        for line in self.lines {
            println!("{line}");
        }
    }

    fn update_cursor(&self) {
        move_to(self.cursor.x + 1, self.cursor.y + 2);
    }

    fn try_read_special(buf: &mut [u8; 2]) -> bool {
        let flags = unsafe { fcntl(STDIN_FILENO, F_GETFL) };
        if flags == -1 {
            return false;
        }

        let new_flags = flags | O_NONBLOCK;
        if unsafe { fcntl(STDOUT_FILENO, F_SETFL, new_flags) } == -1 {
            return false;
        }

        let ret = stdin().read(buf).is_ok();

        if unsafe { fcntl(STDOUT_FILENO, F_SETFL, flags) } == -1 {
            return false;
        }

        ret
    }

    fn header(&self) {
        let filename = "<unnamed file>";

        #[cfg(debug_assertions)]
        {
            let stamp = format!("{}/{} {}", env!("CARGO_PKG_AUTHORS"), env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            print!(
                "{CSI}{BG}{SURFACE_0}{CSI}{FG}{SUBTEXT_0}{filename}{}{CSI}{FG}{OVERLAY_0}{stamp}",
                " ".repeat(self.size.ws_col as usize - (filename.len() + stamp.len()))
            );
        }

        #[cfg(not(debug_assertions))]
        print!(
            "{CSI}{BG}{SURFACE_0}{CSI}{FG}{SUBTEXT_0}{filename}{}",
            " ".repeat(self.size.ws_col as usize - filename.len()),
        );
    }

    fn line_background(&self, is_real: bool) {
        print!(
            "{CSI}{BG}{}{}",
            if is_real { BASE } else { MANTLE },
            " ".repeat(
                if is_real {
                    self.size.ws_col as usize - self.lines[self.lines.len() - 1].len()
                } else {
                    self.size.ws_col as usize
                }
            )
        );
        if !is_real {
            print!("{CSI}{BG}{BASE}");
        }
    }

    fn text_area(&self) {
        print!(
            "{CSI}{BG}{BASE}{}{CSI}{BG}{MANTLE}{}{CSI}{BG}{BASE}{CSI}{FG}{TEXT}",
            " ".repeat(self.size.ws_col as usize),
            " ".repeat((self.size.ws_col * (self.size.ws_row - 2)) as usize)
        );
    }
}

fn show()                      { print!("{CSI}?25h");     }
fn hide()                      { print!("{CSI}?25l");     }
fn alternative_screen()        { print!("{CSI}?1049h");   }
fn normal_screen()             { print!("{CSI}?1049l");   }
fn clear()                     { print!("{CSI}2J");       }
fn move_to(x: usize, y: usize) { print!("{CSI}{y};{x}H"); }
fn move_to_x(p: usize)         { print!("{CSI}{p}G");     }
fn move_left(d: usize)         { print!("{CSI}{d}D");     }
fn move_to_next_line(d: usize) { print!("{CSI}{d}E");     }

