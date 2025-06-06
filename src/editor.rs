// text-editor/src/editor.rs

use libc::{
    fcntl, ioctl, iscntrl, tcgetattr, tcsetattr, winsize,
    BRKINT, CS8, ECHO, F_GETFL, F_SETFL, ICANON, ICRNL, IEXTEN, INPCK, ISIG, ISTRIP, IXON, O_NONBLOCK, OPOST, STDIN_FILENO, STDOUT_FILENO, TCSAFLUSH, TIOCGWINSZ
};

use std::{
    collections::HashMap,
    env::args,
    fs::{create_dir_all, exists, read_to_string, File},
    io::{stdin, stdout, Error, ErrorKind, Read as _, Write as _},
    mem::zeroed,
    path::Path
};

use super::ansi::{
    color::{RESET, FOREGROUND_RED},
    clear, cursor, state,
    CSI
};


static CURSOR_POSITIONS_FILE: &str = "cursor_positions.txt";

static FG: &str = "38;2;";
static BG: &str = "48;2;";

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


pub enum EditorResult {
    Ok(Editor),
    Err(String)
}

impl EditorResult {
    pub fn and_or(self, ok_f: impl FnOnce(Editor), err_f: impl FnOnce(String)) {
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

pub struct Editor {
    file:   Option<(String, bool)>,
    cursor: Cursor,
    size:   winsize,
    lines:  Vec<String>
}

impl Editor {
    pub fn default() -> EditorResult {
        let mut size: winsize = unsafe { zeroed() };

        if unsafe { ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut size) } != 0 {
            return EditorResult::Err(
                format!(
                    "`libc::ioctl` returned \"{}\"",
                    Error::last_os_error()
                )
            );
        }

        let cursor = Cursor { last_x: 0, x: 0, y: 0 };

        let mut args = args();
        let (file, lines) = match args.len() {
            1 => {
                let mut lines = Vec::with_capacity(2048);
                lines.push(String::with_capacity(256));

                (None, lines)
            },
            2 => {
                let filepath        = args.nth(1).unwrap();
                let (lines, exists) = match read_to_string(&filepath) {
                    Ok(string) => {
                        if string.is_empty() {
                            let mut lines = Vec::with_capacity(2048);
                            lines.push(String::with_capacity(256));
                            (lines, true)
                        } else {
                            if !string.is_ascii() {
                                return EditorResult::Err(String::from("non-ascii characters in file are currently not supported"));
                            }

                            (string.lines().map(String::from).collect(), true)
                        }
                    },
                    Err(err) => {
                        if err.kind() == ErrorKind::NotFound {
                            let mut lines = Vec::with_capacity(2048);
                            lines.push(String::with_capacity(256));

                            (lines, false)
                        } else {
                            return EditorResult::Err(format!("`std::fs::read_to_string` returned \"{err}\""));
                        }
                    }
                };

                (Some((filepath, exists)), lines)
            },
            _ => {
                return EditorResult::Err(String::from("multiple files are currently not supported"));
            }
        };

        EditorResult::Ok(Self { file, cursor, size, lines })
    }

    #[expect(clippy::too_many_lines, clippy::cognitive_complexity, reason = "temp")]
    pub fn run(mut self) {
        let mut original_termios = unsafe { zeroed() };

        if unsafe { tcgetattr(STDIN_FILENO, &raw mut original_termios) } != 0 {
            eprintln!(
                "{CSI}{FOREGROUND_RED}error{CSI}{RESET}: `libc::tcgetattr` returned \"{}\"",
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

        if unsafe { tcsetattr(STDIN_FILENO, TCSAFLUSH, &raw const new_termios) } != 0 {
            eprintln!(
                "{CSI}{FOREGROUND_RED}error{CSI}{RESET}: `libc::tcsetattr` returned \"{}\"",
                Error::last_os_error()
            );
            return;
        }

        cursor::hide();
        state::alternative_screen();
        clear::whole_screen();

        cursor::move_to(1, 1);
        self.header();
        self.text_area();

        cursor::move_to(1, 2);
        cursor::show();

        if let Some((filepath, exists)) = &self.file {
            if *exists {
                print!("{}{CSI}0K", self.lines.join(&format!("{CSI}0K{CSI}1E")));

                let file = format!("/home/{}/.local/state/{}/{CURSOR_POSITIONS_FILE}", env!("USER"), env!("CARGO_PKG_NAME"));
                if let Ok(content) = read_to_string(&file) {
                    let map = Self::file_to_cursor_positions(content);
                    if let Some((x, y)) = map.get(filepath) {
                        self.cursor.x = *x;
                        self.cursor.y = *y;
                    }
                }

                self.update_cursor();
            }
        }

        // i will clean this up one day
        loop {
            let _  = stdout.flush();
            let mut buffer = [0u8; 1];
            let mut idklol = [0u8; 2];  // note: do this differently?

            match stdin.read_exact(&mut buffer) {
                Ok(()) => {
                    if unsafe { iscntrl(i32::from(buffer[0])) } == 0 {
                        let c = char::from(buffer[0]);
                        print!("{c}");
                        self.lines[self.cursor.y].insert(self.cursor.x, c);
                        self.cursor.x += 1;

                        if self.cursor.x < self.lines[self.cursor.y].len() {
                            print!("{}", &self.lines[self.cursor.y][self.cursor.x..]);
                            cursor::move_to_x(self.cursor.x + 1);
                        }

                        self.cursor.last_x = self.cursor.x;
                        self.try_make_dirty();
                    } else {
                        match buffer[0] {
                            8 => {  // Ctrl+Backspace
                                if self.cursor.x == 0 {
                                    if self.cursor.y != 0 {
                                        self.backspace();
                                        continue;
                                    }
                                } else {
                                    let left = self.get_left_whitespace_x();

                                    self.lines[self.cursor.y].replace_range(left..self.cursor.x, "");
                                    let old_cursor_x = self.cursor.x;
                                    self.cursor.x = left;
                                    self.update_cursor();
                                    let remainder = &self.lines[self.cursor.y][self.cursor.x..];
                                    print!("{remainder}{}", " ".repeat(old_cursor_x - left));
                                    self.update_cursor();
                                    self.try_make_dirty();
                                }

                                self.cursor.last_x = self.cursor.x;
                            },
                            9 => {  // Tab
                                const TAB: &str = "    ";
                                print!("{TAB}");
                                self.lines[self.cursor.y].insert_str(self.cursor.x, TAB);
                                self.cursor.x += 4;

                                if self.cursor.x < self.lines[self.cursor.y].len() {
                                    print!("{}", &self.lines[self.cursor.y][self.cursor.x..]);
                                    cursor::move_to_x(self.cursor.x + 1);
                                }

                                self.cursor.last_x = self.cursor.x;
                                self.try_make_dirty();
                            },
                            13 => {  // Enter
                                self.cursor.y += 1;
                                self.lines.insert(self.cursor.y, String::with_capacity(256));

                                if self.cursor.x < self.lines[self.cursor.y - 1].len() {
                                    print!("{CSI}0K");
                                    let remainder = self.lines[self.cursor.y - 1].drain(self.cursor.x..).collect::<String>();
                                    self.lines[self.cursor.y].push_str(&remainder);
                                }

                                cursor::move_to_next_line(1);
                                self.cursor.x = 0;
                                print!("{}", self.lines[self.cursor.y..].join(&format!("{CSI}0K{CSI}1E")));
                                self.line_background(true);
                                self.update_cursor();

                                self.cursor.last_x = self.cursor.x;
                                self.try_make_dirty();
                            },
                            19 => {  // Ctrl+S
                                let Some((filepath, _)) = &self.file else { continue; };

                                if let Some(parent) = Path::new(&filepath).parent() {
                                    create_dir_all(parent).unwrap();
                                }

                                let mut file    = File::create(filepath).unwrap();
                                let mut content = self.lines.join("\n");
                                self.file.as_mut().unwrap().1 = true;

                                if self.lines.len() == 1 && self.lines[0].is_empty() {
                                    file.set_len(0).unwrap();
                                    continue;
                                }

                                if self.lines[self.lines.len() - 1].is_empty() {
                                    content.push('\n');
                                }
                                file.write_all(content.as_bytes()).unwrap();
                            },
                            27 => {
                                if !Self::try_read_special(&mut idklol) {  // Escape
                                    print!("{CSI}{RESET}");
                                    state::normal_screen();
                                    let _ = stdout.flush();
                                    unsafe { tcsetattr(STDIN_FILENO, TCSAFLUSH, &raw const original_termios) };
                                    break;
                                }

                                match idklol[0] {
                                    91 => {
                                        match idklol[1] {
                                            49 => {
                                                let mut lolidk = [0u8; 3];

                                                if !Self::try_read_special(&mut lolidk) {
                                                    print!("{CSI}{RESET}");
                                                    state::normal_screen();
                                                    let _ = stdout.flush();
                                                    unsafe { tcsetattr(STDIN_FILENO, TCSAFLUSH, &raw const original_termios) };
                                                    eprintln!("{}: todo: buf={buffer:?} ; idklol={idklol:?}", line!());
                                                    break;
                                                }

                                                if lolidk[0] != 59 || lolidk[1] != 53 {
                                                    print!("{CSI}{RESET}");
                                                    state::normal_screen();
                                                    let _ = stdout.flush();
                                                    unsafe { tcsetattr(STDIN_FILENO, TCSAFLUSH, &raw const original_termios) };
                                                    eprintln!("{}: todo: buf={buffer:?} ; idklol={idklol:?} ; lolidk={lolidk:?}", line!());
                                                    break;
                                                }

                                                match lolidk[2] {
                                                    65 => {  // Ctrl+ArrowUp
                                                        self.up();
                                                    },
                                                    66 => {  // Ctrl+ArrowDown
                                                        self.down();
                                                    },
                                                    67 => {  // Ctrl+ArrowRight
                                                        let len = self.lines[self.cursor.y].len();
                                                        if self.cursor.x == len {
                                                            if self.cursor.y != self.lines.len() - 1 {
                                                                self.cursor.x  = 0;
                                                                self.cursor.y += 1;
                                                                self.update_cursor();
                                                            }
                                                        } else {
                                                            let right = self.get_right_whitespace_x();

                                                            if right == len {
                                                                self.cursor.x  = len;
                                                            } else {
                                                                self.cursor.x += right;
                                                            }

                                                            self.update_cursor();
                                                        }

                                                        self.cursor.last_x = self.cursor.x;
                                                    },
                                                    68 => {  // Ctrl+ArrowLeft
                                                        if self.cursor.x == 0 {
                                                            if self.cursor.y != 0 {
                                                                self.cursor.y -= 1;
                                                                self.cursor.x  = self.lines[self.cursor.y].len();
                                                                self.update_cursor();
                                                            }
                                                        } else {
                                                            let left = self.get_left_whitespace_x();

                                                            self.cursor.x = left;
                                                            self.update_cursor();
                                                            self.cursor.last_x = self.cursor.x;
                                                        }
                                                    },
                                                    70 => {  // Ctrl+End
                                                        let last = self.lines.len() - 1;
                                                        if self.cursor.y == last {
                                                            self.cursor.last_x = self.cursor.x;
                                                        } else {
                                                            self.cursor.y = last;
                                                            self.cursor.x = self.cursor.x.max(self.cursor.last_x).min(self.lines[self.cursor.y].len());
                                                            self.update_cursor();
                                                        }
                                                    },
                                                    72 => {  // Ctrl+Home
                                                        if self.cursor.y == 0 {
                                                            self.cursor.last_x = self.cursor.x;
                                                        } else {
                                                            self.cursor.y = 0;
                                                            self.cursor.x = self.cursor.x.max(self.cursor.last_x).min(self.lines[self.cursor.y].len());
                                                            self.update_cursor();
                                                        }
                                                    },
                                                    _ => {
                                                        print!("{CSI}{RESET}");
                                                        state::normal_screen();
                                                        let _  = stdout.flush();
                                                        unsafe { tcsetattr(STDIN_FILENO, TCSAFLUSH, &raw const original_termios) };
                                                        eprintln!("{}: todo: buf={buffer:?} ; idklol={idklol:?} ; lolidk={lolidk:?}", line!());
                                                        break;
                                                    }
                                                }
                                            },
                                            51 => {
                                                let mut lolidk = [0u8; 3];

                                                if !Self::try_read_special(&mut lolidk) {
                                                    print!("{CSI}{RESET}");
                                                    state::normal_screen();
                                                    let _ = stdout.flush();
                                                    unsafe { tcsetattr(STDIN_FILENO, TCSAFLUSH, &raw const original_termios) };
                                                    eprintln!("{}: todo: buf={buffer:?} ; idklol={idklol:?} ; lolidk={lolidk:?}", line!());
                                                    break;
                                                }

                                                match lolidk[0] {
                                                    59 => {
                                                        if lolidk[2] != 126 {
                                                            print!("{CSI}{RESET}");
                                                            state::normal_screen();
                                                            let _ = stdout.flush();
                                                            unsafe { tcsetattr(STDIN_FILENO, TCSAFLUSH, &raw const original_termios) };
                                                            eprintln!("{}: todo: buf={buffer:?} ; idklol={idklol:?} ; lolidk={lolidk:?}", line!());
                                                            break;
                                                        }

                                                        match lolidk[1] {
                                                            51 => {  // Alt+Delete
                                                                let len = self.lines[self.cursor.y].len();
                                                                if self.cursor.x != len {
                                                                    self.lines[self.cursor.y].truncate(self.cursor.x);
                                                                    let count = len - self.cursor.x;
                                                                    print!("{}", " ".repeat(count));
                                                                    self.update_cursor();
                                                                    self.try_make_dirty();
                                                                }

                                                                self.cursor.last_x = self.cursor.x;
                                                            },
                                                            53 => {  // Ctrl+Delete
                                                                if self.cursor.x == self.lines[self.cursor.y].len() {
                                                                    if self.cursor.y != self.lines.len() {
                                                                        self.delete();
                                                                        continue;
                                                                    }
                                                                } else {
                                                                    let right = self.get_right_whitespace_x();

                                                                    if right == self.lines[self.cursor.y].len() {
                                                                        let count = self.lines[self.cursor.y].len() - self.cursor.x;
                                                                        self.lines[self.cursor.y].truncate(self.cursor.x);
                                                                        print!("{}", " ".repeat(count));
                                                                    } else {
                                                                        let remainder = String::from(&self.lines[self.cursor.y][self.cursor.x + right..]);
                                                                        let mut count = self.lines[self.cursor.y].len();
                                                                        self.lines[self.cursor.y].replace_range(self.cursor.x..self.cursor.x + right, "");
                                                                        count -= self.lines[self.cursor.y].len();
                                                                        print!("{remainder}{}", " ".repeat(count));
                                                                    }

                                                                    self.update_cursor();
                                                                    self.try_make_dirty();
                                                                }

                                                                self.cursor.last_x = self.cursor.x;
                                                            },
                                                            _ => {
                                                                print!("{CSI}{RESET}");
                                                                state::normal_screen();
                                                                let _ = stdout.flush();
                                                                unsafe { tcsetattr(STDIN_FILENO, TCSAFLUSH, &raw const original_termios) };
                                                                eprintln!("{}: todo: buf={buffer:?} ; idklol={idklol:?} ; lolidk={lolidk:?}", line!());
                                                                break;
                                                            }
                                                        }
                                                    },
                                                    126 => {  // Delete
                                                        if lolidk[1] != 0 || lolidk[2] != 0 {
                                                            print!("{CSI}{RESET}");
                                                            state::normal_screen();
                                                            let _ = stdout.flush();
                                                            unsafe { tcsetattr(STDIN_FILENO, TCSAFLUSH, &raw const original_termios) };
                                                            eprintln!("{}: todo: buf={buffer:?} ; idklol={idklol:?} ; lolidk={lolidk:?}", line!());
                                                            break;
                                                        }

                                                        self.delete();
                                                    },
                                                    _ => {
                                                        print!("{CSI}{RESET}");
                                                        state::normal_screen();
                                                        let _ = stdout.flush();
                                                        unsafe { tcsetattr(STDIN_FILENO, TCSAFLUSH, &raw const original_termios) };
                                                        eprintln!("{}: todo: buf={buffer:?} ; idklol={idklol:?} ; lolidk={lolidk:?}", line!());
                                                        break;
                                                    }
                                                }
                                            },
                                            65 => {  // ArrowUp/ScrollUp
                                                self.up();
                                            },
                                            66 => {  // ArrowDown/ScrollDown
                                                self.down();
                                            },
                                            67 => {  // ArrowRight
                                                if self.cursor.x == self.lines[self.cursor.y].len() {
                                                    if self.cursor.y < self.lines.len() - 1 {
                                                        cursor::move_to_next_line(1);
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
                                            70 => {  // End
                                                let len = self.lines[self.cursor.y].len();
                                                if self.cursor.x != len {
                                                    self.cursor.x = len;
                                                    self.update_cursor();
                                                }

                                                self.cursor.last_x = self.cursor.x;
                                            },
                                            72 => {  // Home
                                                if self.cursor.x != 0 {
                                                    self.cursor.x = 0;
                                                    self.update_cursor();
                                                }

                                                self.cursor.last_x = self.cursor.x;
                                            },
                                            _ => {
                                                print!("{CSI}{RESET}");
                                                state::normal_screen();
                                                let _ = stdout.flush();
                                                unsafe { tcsetattr(STDIN_FILENO, TCSAFLUSH, &raw const original_termios) };
                                                eprintln!("{}: todo: buf={buffer:?} ; idklol={idklol:?}", line!());
                                                break;
                                            }
                                        }
                                    },
                                    127 => {  // Alt+Backspace
                                        let mut lolidk = [0u8; 3];

                                        if Self::try_read_special(&mut lolidk) {
                                            print!("{CSI}{RESET}");
                                            state::normal_screen();
                                            let _ = stdout.flush();
                                            unsafe { tcsetattr(STDIN_FILENO, TCSAFLUSH, &raw const original_termios) };
                                            eprintln!("{}: todo: buf={buffer:?} ; idklol={idklol:?} ; lolidk={lolidk:?}", line!());
                                            break;
                                        }

                                        if self.cursor.x != 0 {
                                            self.lines[self.cursor.y].replace_range(0..self.cursor.x, "");
                                            cursor::move_to_x(0);
                                            print!("{}{}", self.lines[self.cursor.y], " ".repeat(self.cursor.x));
                                            self.cursor.x = 0;
                                            self.update_cursor();
                                            self.try_make_dirty();
                                        }

                                        self.cursor.last_x = self.cursor.x;
                                    },
                                    _ => {
                                        print!("{CSI}{RESET}");
                                        state::normal_screen();
                                        let _ = stdout.flush();
                                        unsafe { tcsetattr(STDIN_FILENO, TCSAFLUSH, &raw const original_termios) };
                                        eprintln!("{}: todo: buf={buffer:?} ; idklol={idklol:?}", line!());
                                        break;
                                    }
                                }
                            },
                            127 => {  // Backspace
                                self.backspace();
                            },
                            _ => {
                                #[cfg(debug_assertions)]
                                print!("{CSI}{FG}{SURFACE_1}[{}]{CSI}{RESET}", buffer[0]);
                            }
                        }
                    }
                },
                Err(err) => {
                    print!("{CSI}{RESET}");
                    state::normal_screen();
                    let _ = stdout.flush();
                    unsafe { tcsetattr(STDIN_FILENO, TCSAFLUSH, &raw const original_termios) };
                    eprintln!("{CSI}{FOREGROUND_RED}error{CSI}{RESET}: `Stdin::read_exact returned \"{err}\"");
                    break;
                }
            }
        }

        self.save_cursor_pos();
    }

    // used to prevent the cursor position from caching for unsaved files
    fn try_make_dirty(&mut self) {
        if let Some((_, exists)) = self.file.as_mut() {
            *exists = false;
        }
    }

    fn save_cursor_pos(self) {
        let dir      = format!("/home/{}/.local/state/{}", env!("USER"), env!("CARGO_PKG_NAME"));
        let filepath = format!("{dir}/{CURSOR_POSITIONS_FILE}");
        create_dir_all(dir).unwrap();

        if !exists(&filepath).unwrap() {
            if let Some((name, exists)) = &self.file {
                if !exists { return; }

                let map      = HashMap::from([(name.clone(), (self.cursor.x, self.cursor.y))]);
                let content  = Self::cursor_positions_to_file(map);
                let mut file = File::create(filepath).unwrap();
                file.write_all(content.as_bytes()).unwrap();
            }

            return;
        }

        if let Some((name, exists)) = &self.file {
            if !exists { return; }

            let content = read_to_string(&filepath).unwrap();
            let mut map = Self::file_to_cursor_positions(content);
            map.insert(name.clone(), (self.cursor.x, self.cursor.y));
            let new_content = Self::cursor_positions_to_file(map);
            let mut file    = File::create(filepath).unwrap();
            file.write_all(new_content.as_bytes()).unwrap();
        }
    }

    fn cursor_positions_to_file(map: HashMap<String, (usize, usize)>) -> String {
        let mut string = String::with_capacity(8192);

        for (filepath, (x, y)) in map {
            string.push_str(&format!("{filepath}:{x},{y}\n"));
        }

        string
    }

    fn file_to_cursor_positions(string: String) -> HashMap<String, (usize, usize)> {
        let mut map = HashMap::with_capacity(512);

        for line in string.lines() {
            let colon    = line.rfind(':').unwrap();
            let comma    = line.rfind(',').unwrap();
            let filepath = &line[..colon];
            let x        = line[colon + 1..comma].parse::<usize>().unwrap();
            let y        = line[comma + 1..     ].parse::<usize>().unwrap();
            map.insert(String::from(filepath), (x, y));
        }

        map
    }

    fn backspace(&mut self) {
        if self.cursor.x == 0 {
            if self.cursor.y == 0 {
                self.cursor.last_x = self.cursor.x;
                return;
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
                cursor::move_to(len + 1, self.cursor.y + 2);
                print!("{line}");
                cursor::move_to(len + 1, self.cursor.y + 2);
                self.cursor.x = len;
            }
        } else if self.cursor.x == self.lines[self.cursor.y].len() {
            cursor::move_left(1);
            print!(" ");
            cursor::move_left(1);
            self.lines[self.cursor.y].pop();
            self.cursor.x -= 1;
        } else {
            cursor::move_left(1);
            print!("{} ", &self.lines[self.cursor.y][self.cursor.x..]);
            cursor::move_to_x(self.cursor.x);
            self.cursor.x -= 1;
            self.lines[self.cursor.y].remove(self.cursor.x);
        }

        self.cursor.last_x = self.cursor.x;
        self.try_make_dirty();
    }

    fn delete(&mut self) {
        let line_count = self.lines.len();
        if self.cursor.x == self.lines[self.cursor.y].len() {
            if self.cursor.y == line_count - 1 {
                self.cursor.last_x = self.cursor.x;
                return;
            }

            if self.lines[self.cursor.y].is_empty() {
                self.lines.remove(self.cursor.y);

                print!("{}", self.lines[self.cursor.y..].join(&format!("{CSI}0K{CSI}1E")));
            } else {
                let len  = self.lines[self.cursor.y].len();
                let line = self.lines.remove(self.cursor.y + 1);
                self.lines[self.cursor.y].push_str(&line);

                print!("{}{CSI}0K{CSI}1E{}", &self.lines[self.cursor.y][len..], self.lines[self.cursor.y + 1..].join(&format!("{CSI}0K{CSI}1E")));
            }

            if self.cursor.y != self.lines.len() {
                print!("{CSI}0K{CSI}1E");
            }

            self.line_background(false);
            self.update_cursor();
        } else if self.cursor.x == 0 {
            self.lines[self.cursor.y].remove(0);
            print!("{} ", self.lines[self.cursor.y]);
            cursor::move_to_x(0);
        } else {
            self.lines[self.cursor.y].remove(self.cursor.x);
            print!("{} ", &self.lines[self.cursor.y][self.cursor.x..]);
            cursor::move_to_x(self.cursor.x + 1);
        }

        self.cursor.last_x = self.cursor.x;
        self.try_make_dirty();
    }

    fn up(&mut self) {
        if self.cursor.y == 0 {
            if self.cursor.x != 0 {
                self.cursor.x = 0;
                cursor::move_to_x(0);
            }

            self.cursor.last_x = self.cursor.x;
            return;
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
    }

    fn down(&mut self) {
        if self.cursor.y == self.lines.len() - 1 {
            let len = self.lines[self.cursor.y].len();
            if self.cursor.x != len {
                self.cursor.x = len;
                cursor::move_to_x(self.cursor.x + 1);
            }

            self.cursor.last_x = self.cursor.x;
            return;
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
    }

    fn get_right_whitespace_x(&self) -> usize {
        if self.lines[self.cursor.y].chars().nth(self.cursor.x).unwrap().is_whitespace() {
            let idk = self.lines
                [self.cursor.y]
                [self.cursor.x..]
                .find(|c: char| !c.is_whitespace())
                .unwrap_or(self.lines[self.cursor.y].len());

            self.lines
                [self.cursor.y]
                [self.cursor.x + idk..]
                    .find(char::is_whitespace)
                    .map_or(self.lines[self.cursor.y].len(), |x| x + idk)
        } else {
            self.lines
                [self.cursor.y]
                [self.cursor.x..]
                .find(char::is_whitespace)
                .map_or(self.lines[self.cursor.y].len(), |x| x)
        }
    }

    fn get_left_whitespace_x(&self) -> usize {
        if self.lines[self.cursor.y].chars().nth(self.cursor.x - 1).unwrap().is_whitespace() {
            self.lines
                [self.cursor.y]
                [
                    ..
                    self.lines
                        [self.cursor.y]
                        [..self.cursor.x - 1]
                        .rfind(|c: char| !c.is_whitespace())
                        .unwrap_or(0)
                ]
                    .rfind(char::is_whitespace)
                    .map_or(0, |x| x + 1)
        } else {
            self.lines
                [self.cursor.y]
                [..self.cursor.x - 1]
                .rfind(char::is_whitespace)
                .map_or(0, |x| x + 1)
        }
    }

    fn update_cursor(&self) {
        cursor::move_to(self.cursor.x + 1, self.cursor.y + 2);
    }

    fn try_read_special<const N: usize>(buf: &mut [u8; N]) -> bool {
        let flags = unsafe { fcntl(STDIN_FILENO, F_GETFL) };
        if flags == -1 {
            eprintln!("{}: fcntl error", line!());
            return false;
        }

        let new_flags = flags | O_NONBLOCK;
        if unsafe { fcntl(STDOUT_FILENO, F_SETFL, new_flags) } == -1 {
            eprintln!("{}: fcntl error", line!());
            return false;
        }

        let ret = stdin().read(buf).is_ok();

        if unsafe { fcntl(STDOUT_FILENO, F_SETFL, flags) } == -1 {
            eprintln!("{}: fcntl error", line!());
            return false;
        }

        ret
    }

    fn header(&self) {
        let filename = if let Some((filepath, _)) = &self.file {
            filepath.clone()
        } else {
            String::from("<unnamed file>")
        };

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

impl Drop for Editor {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        {
            println!("--- file");
            for line in &self.lines {
            println!("{CSI}{FOREGROUND_RED}|{CSI}{RESET}{line}{CSI}{FOREGROUND_RED}|{CSI}{RESET}");
            }
            println!("--- EOF");
        }
    }
}

