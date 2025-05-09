// text-editor/src/main.rs

use libc::{
    ioctl, iscntrl, tcgetattr, tcsetattr, winsize,
    BRKINT, CS8, ECHO, ICANON, ICRNL, IEXTEN, INPCK, ISIG, ISTRIP, IXON, OPOST, STDIN_FILENO, STDOUT_FILENO, TCSAFLUSH, TIOCGWINSZ
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

// Catppuccin Mocha
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

struct Editor {
    size: winsize
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


        EditorResult::Ok(Self { size })
    }

    fn run(self) {
        let mut original_termios = unsafe { zeroed() };

        if unsafe { tcgetattr(STDIN_FILENO, &mut original_termios) } != 0 {
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

        if unsafe { tcsetattr(STDIN_FILENO, TCSAFLUSH, &new_termios) } != 0 {
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
                        print!("{}", char::from(buffer[0]));
                    } else {
                        match buffer[0] {
                            13 => {  // CR
                                move_down(1);
                                self.text_line();
                                let _ = stdout.flush();
                            },
                            27 => {  // Escape
                                // TODO: also other stuff report 27,
                                //       like the start of sequences
                                //       for PageUp/Delete/...
                                break;
                            },
                            127 => {  // Backspace
                                // TODO: check if there even is something to erase
                                move_left(1);
                                print!(" ");
                                move_left(1);
                                let _ = stdout.flush();
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
                    unsafe { tcsetattr(STDIN_FILENO, TCSAFLUSH, &original_termios) };
                    eprintln!("{CSI}{RED}error{CSI}{RESET}: `Stdin::read_exact returned \"{err}\"");
                    return;
                }
            }
        }

        print!("{CSI}{RESET}");
        normal_screen();
        unsafe { tcsetattr(STDIN_FILENO, TCSAFLUSH, &original_termios) };
    }

    fn header(&self) {
        let filename = "<unnamed file>";

        #[cfg(debug_assertions)]
        {
            let stamp = format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
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

    fn text_line(&self) {
        print!(
            "{CSI}{BG}{BASE}{}",
            " ".repeat(self.size.ws_col as usize)
        );
        move_to_line_start();
    }

    fn text_area(&self) {
        print!(
            "{CSI}{BG}{BASE}{}{CSI}{BG}{MANTLE}{}{CSI}{BG}{BASE}{CSI}{FG}{TEXT}",
            " ".repeat(self.size.ws_col as usize),
            " ".repeat((self.size.ws_col * (self.size.ws_row - 2)) as usize)
        );
    }
}

fn show()                  { print!("{CSI}?25h");     }
fn hide()                  { print!("{CSI}?25l");     }
fn alternative_screen()    { print!("{CSI}?1049h");   }
fn normal_screen()         { print!("{CSI}?1049l");   }
fn clear()                 { print!("{CSI}2J");       }
fn move_to_line_start()    { print!("{CSI}1G");       }
fn move_to(x: u16, y: u16) { print!("{CSI}{y};{x}H"); }
fn move_left(d: u16)       { print!("{CSI}{d}D");     }
fn move_down(d: u16)       { print!("{CSI}{d}E");     }

