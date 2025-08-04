// text-editor/src/editor.rs

use libc::{
    self,
    fcntl, iscntrl, tcgetattr, tcsetattr,
    CREAD, CS8, F_GETFL, F_SETFL, O_NONBLOCK, STDIN_FILENO, STDOUT_FILENO, TCSAFLUSH
};

use std::{
    io::{self, stdin, Stdin, stdout, Stdout, Read as _, Write as _},
    mem::zeroed
};

use crate::ansi::{cursor, state, clear};


#[expect(dead_code, reason = "temporarily only printing")]
#[derive(Debug)]
enum Command {
    Printable(u8),
    Escape,
    Tab,
    Backspace,
    Delete,
    Enter,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    ArrowUp,
    ArrowDown,
    ArrowRight,
    ArrowLeft,
    ShiftTab,
    ShiftEnter,
    ShiftHome,
    ShiftEnd,
    ShiftArrowUp,
    ShiftArrowDown,
    ShiftArrowRight,
    ShiftArrowLeft,
    CtrlA,
    CtrlC,
    CtrlQ,
    CtrlR,
    CtrlS,
    CtrlV,
    CtrlX,
    CtrlY,
    CtrlZ,
    CtrlBackspace,
    CtrlDelete,
    CtrlHome,
    CtrlEnd,
    CtrlArrowUp,
    CtrlArrowDown,
    CtrlArrowRight,
    CtrlArrowLeft,
    MouseLeftPress      (u16, u16),
    MouseLeftDrag       (u16, u16),
    MouseLeftRelease    (u16, u16),
    MouseRightPress     (u16, u16),
    MouseRightDrag      (u16, u16),
    MouseRightRelease   (u16, u16),
    MouseMiddlePress    (u16, u16),
    MouseMiddleDrag     (u16, u16),
    MouseMiddleRelease  (u16, u16),
    MouseForwardPress   (u16, u16),
    MouseForwardDrag    (u16, u16),
    MouseForwardRelease (u16, u16),
    MouseBackPress      (u16, u16),
    MouseBackDrag       (u16, u16),
    MouseBackRelease    (u16, u16),
    MouseHover          (u16, u16),
    ScrollUp            (u16, u16),
    ScrollDown          (u16, u16),
    Exit,
    Nop,
    Error(io::Error)
}

struct Cursor {
    last_x: usize,
    x:      usize,
    y:      usize
}

pub struct Editor {
    cursor: Cursor,
    lines:  Vec<String>
}

impl Editor {
    pub fn new() -> Self {
        let     cursor = Cursor { last_x: 0, x: 0, y: 0 };
        let mut lines  = Vec::with_capacity(2048);

        lines.push(String::with_capacity(512));

        Self { cursor, lines }
    }

    #[expect(clippy::too_many_lines)]
    pub fn run(mut self) -> io::Result<()> {
        let mut error  = None;
        let mut buffer = [0u8;  1];
        let mut trail  = [0u8; 32];
        let     stdin  = stdin();
        let mut stdout = stdout();

        let original_termios = prepare_terminal(&mut stdout)?;

        loop {
            match blocking_read_to_command(&stdin, &mut buffer, &mut trail) {
                Command::Escape => {
                    break;
                },
                Command::Error(err) => {
                    error = Some(err);
                    break;
                },
                Command::Printable(byte) => {
                    let character = byte as char;
                    print!("{character}");
                    if self.cursor.x < self.lines[self.cursor.y].len() {
                        print!("{}", &self.lines[self.cursor.y][self.cursor.x..]);
                        cursor::move_to_x(self.cursor.x + 2);
                    }
                    stdout.flush()?;
                    self.lines[self.cursor.y].insert(self.cursor.x, character);
                    self.cursor.x      += 1;
                    self.cursor.last_x  = self.cursor.x;
                },
                Command::Enter => {
                    cursor::move_to_next_line(1);
                    stdout.flush()?;
                    self.lines.push(String::with_capacity(512));
                    self.cursor.y      += 1;
                    self.cursor.x       = 0;
                    self.cursor.last_x  = 0;
                },
                Command::ArrowLeft => {
                    if self.cursor.x == 0 {
                        if self.cursor.y == 0 {
                            self.cursor.last_x = 0;
                            continue;
                        }
                        self.cursor.y -= 1;
                        self.cursor.x  = self.lines[self.cursor.y].len();
                        cursor::move_to(self.cursor.x + 1, self.cursor.y + 1);
                    } else {
                        cursor::move_left(1);
                        self.cursor.x -= 1;
                    }
                    self.cursor.last_x = self.cursor.x;
                    stdout.flush()?;
                },
                Command::ArrowRight => {
                    if self.cursor.x == self.lines[self.cursor.y].len() {
                        if self.cursor.y == self.lines.len() - 1 {
                            self.cursor.last_x = self.cursor.x;
                            continue;
                        }
                        self.cursor.y += 1;
                        self.cursor.x  = 0;
                        cursor::move_to_next_line(1);
                    } else {
                        cursor::move_right(1);
                        self.cursor.x += 1;
                    }
                    self.cursor.last_x = self.cursor.x;
                    stdout.flush()?;
                },
                Command::ArrowUp => {
                    if self.cursor.y == 0 {
                        if self.cursor.x == 0 {
                            continue;
                        }
                        self.cursor.x      = 0;
                        self.cursor.last_x = 0;
                        cursor::move_to_x(1);
                    } else {
                        self.cursor.x  = self.cursor.last_x;
                        self.cursor.y -= 1;
                        self.cursor.x  = self.cursor.x.min(self.lines[self.cursor.y].len());
                        cursor::move_to(self.cursor.x + 1, self.cursor.y + 1);
                    }
                    stdout.flush()?;
                },
                Command::ArrowDown => {
                    if self.cursor.y == self.lines.len() - 1 {
                        if self.cursor.x == self.lines[self.cursor.y].len() {
                            continue;
                        }
                        self.cursor.x      = self.lines[self.cursor.y].len();
                        self.cursor.last_x = self.cursor.x;
                        cursor::move_to_x(self.cursor.x + 1);
                    } else {
                        self.cursor.x  = self.cursor.last_x;
                        self.cursor.y += 1;
                        self.cursor.x  = self.cursor.x.min(self.lines[self.cursor.y].len());
                        cursor::move_to(self.cursor.x + 1, self.cursor.y + 1);
                    }
                    stdout.flush()?;
                },
                Command::Home => {
                    self.cursor.last_x = 0;
                    if self.cursor.x == 0 {
                        continue;
                    }
                    self.cursor.x = 0;
                    cursor::move_to_x(1);
                    stdout.flush()?;
                },
                Command::End => {
                    self.cursor.last_x = self.lines[self.cursor.y].len();
                    if self.cursor.x == self.lines[self.cursor.y].len() {
                        continue;
                    }
                    self.cursor.x = self.lines[self.cursor.y].len();
                    cursor::move_to_x(self.cursor.x + 1);
                    stdout.flush()?;
                },
                Command::CtrlHome => {
                    if self.cursor.y == 0 {
                        if self.cursor.x != 0 {
                            self.cursor.x = 0;
                            cursor::move_to_x(1);
                            stdout.flush()?;
                        }
                        self.cursor.last_x = 0;
                        continue;
                    }
                    cursor::move_up(self.cursor.y);
                    stdout.flush()?;
                    self.cursor.y = 0;
                },
                Command::CtrlEnd => {
                    if self.cursor.y == self.lines.len() - 1 {
                        if self.cursor.x != self.lines[self.cursor.y].len() {
                            self.cursor.x = self.lines[self.cursor.y].len();
                            cursor::move_to_x(self.cursor.x + 1);
                            stdout.flush()?;
                        }
                        self.cursor.last_x = self.cursor.x;
                        continue;
                    }
                    cursor::move_down(self.lines.len() - self.cursor.y - 1);
                    stdout.flush()?;
                    self.cursor.y = self.lines.len() - 1;
                }
                #[cfg(debug_assertions)]
                other => {
                    print!("{other:?}, buffer={buffer:?}, trail={trail:?}            ");
                    cursor::move_to_next_line(1);
                    print!("{}            ", str::from_utf8(&trail).unwrap());
                    cursor::move_to_next_line(2);
                    stdout.flush()?;
                },
                #[cfg(not(debug_assertions))]
                _ => ()
            }

            buffer = [0u8;  1];
            trail  = [0u8; 32];
        }

        restore_terminal(original_termios, &mut stdout)?;

        println!("----- file content -----");
        for line in self.lines {
            println!("{line}");
        }
        println!("------------------------");

        if let Some(err) = error {
            eprintln!("{err}");
        }

        Ok(())
    }
}

fn get_termios() -> io::Result<libc::termios> {
    let mut termios = unsafe { zeroed() };

    if unsafe { tcgetattr(STDIN_FILENO, &raw mut termios) } != 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(termios)
}

const fn raw_termios(mut termios: libc::termios) -> libc::termios {
    termios.c_iflag = 0;           // &= !(BRKINT | ICRNL | INPCK | ISTRIP | IXON | INLCR | IGNCR);
    termios.c_oflag = 0;           // &= !(OPOST);
    termios.c_cflag = CREAD | CS8; // |= CREAD | CS8;
    termios.c_lflag = 0;           // &= !(ECHO | ICANON | IEXTEN | ISIG);

    termios
}

fn set_termios(termios: libc::termios) -> io::Result<()> {
    if unsafe { tcsetattr(STDIN_FILENO, TCSAFLUSH, &raw const termios) } != 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}

fn prepare_terminal(stdout: &mut Stdout) -> io::Result<libc::termios> {
    let original_termios = get_termios()?;
    let      new_termios = raw_termios(original_termios);

    set_termios(new_termios)?;
    state::alternative_screen();
    state::enable_mouse();
    clear::whole_screen();
    cursor::move_to(1, 1);

    stdout.flush()?;

    Ok(original_termios)
}

fn restore_terminal(termios: libc::termios, stdout: &mut Stdout) -> io::Result<()> {
    state::normal_screen();
    state::disable_mouse();
    set_termios(termios)?;

    stdout.flush()?;

    Ok(())
}

enum MouseEventParsingStage {
    Action,
    X,
    Y,
    Finished(u8)
}

#[expect(clippy::too_many_lines)]
fn parse_mouse_event(bytes: &[u8]) -> Option<Command> {
    let mut action = 0;
    let mut x      = 0;
    let mut y      = 0;
    let mut stage  = MouseEventParsingStage::Action;

    for byte in bytes {
        match stage {
            MouseEventParsingStage::Action => {
                if *byte == 59 {
                    stage = MouseEventParsingStage::X;
                } else {
                    action = action * 10 + u16::from(byte - 48);
                }
            },
            MouseEventParsingStage::X => {
                if *byte == 59 {
                    stage = MouseEventParsingStage::Y;
                } else {
                    x = x * 10 + u16::from(byte - 48);
                }
            },
            MouseEventParsingStage::Y => {
                if *byte == 77 || *byte == 109 {
                    stage = MouseEventParsingStage::Finished(*byte);
                    break;
                }

                y = y * 10 + u16::from(byte - 48);
            },
            MouseEventParsingStage::Finished(_) => ()
        }
    }

    if let MouseEventParsingStage::Finished(last_byte) = stage {
        match action {
            0 => {
                match last_byte {
                    77  => { return Some(Command::MouseLeftPress   (x, y)); },
                    109 => { return Some(Command::MouseLeftRelease (x, y)); },
                    _   => ()
                }
            },
            1 => {
                match last_byte {
                    77  => { return Some(Command::MouseMiddlePress   (x, y)); },
                    109 => { return Some(Command::MouseMiddleRelease (x, y)); },
                    _   => ()
                }
            },
            2 => {
                match last_byte {
                    77  => { return Some(Command::MouseRightPress   (x, y)); },
                    109 => { return Some(Command::MouseRightRelease (x, y)); },
                    _   => ()
                }
            },
            32 => {
                if last_byte == 77 {
                    return Some(Command::MouseLeftDrag(x, y));
                }
            },
            33 => {
                if last_byte == 77 {
                    return Some(Command::MouseMiddleDrag(x, y));
                }
            },
            34 => {
                if last_byte == 77 {
                    return Some(Command::MouseRightDrag(x, y));
                }
            },
            35 => {
                if last_byte == 77 {
                    return Some(Command::MouseHover(x, y));
                }
            },
            64 => {
                if last_byte == 77 {
                    return Some(Command::ScrollUp(x, y));
                }
            },
            65 => {
                if last_byte == 77 {
                    return Some(Command::ScrollDown(x, y));
                }
            },
            128 => {
                match last_byte {
                    77  => { return Some(Command::MouseBackPress   (x, y)); },
                    109 => { return Some(Command::MouseBackRelease (x, y)); },
                    _   => ()
                }
            },
            129 => {
                match last_byte {
                    77  => { return Some(Command::MouseForwardPress   (x, y)); },
                    109 => { return Some(Command::MouseForwardRelease (x, y)); },
                    _   => ()
                }
            },
            160 => {
                if last_byte == 77 {
                    return Some(Command::MouseBackDrag(x, y));
                }
            },
            161 => {
                if last_byte == 77 {
                    return Some(Command::MouseForwardDrag(x, y));
                }
            },
            _ => ()
        }
    }

    None
}

// TODO: bytes of more than just one event could be read at one time, this is currently not handled
//       i could just read more and more conditionally based on previous bytes, instead of always 32
#[expect(clippy::too_many_lines)]
fn blocking_read_to_command<const N: usize>(mut stdin: &Stdin, buffer: &mut [u8; 1], trail: &mut [u8; N]) -> Command {
    match stdin.read_exact(buffer) {
        Ok(()) => {
            let cntrl = unsafe { iscntrl(i32::from(buffer[0])) };
            match cntrl {
                0 => { return Command::Printable(buffer[0]); },
                2 => {
                    match buffer[0] {
                        1  => { return Command::CtrlA;         },
                        3  => { return Command::CtrlC;         },
                        8  => { return Command::CtrlBackspace; },
                        9  => { return Command::Tab;           },
                        13 => { return Command::Enter;         },
                        17 => { return Command::CtrlQ;         }
                        18 => { return Command::CtrlR;         },
                        19 => { return Command::CtrlS;         },
                        22 => { return Command::CtrlV;         },
                        24 => { return Command::CtrlX;         },
                        25 => { return Command::CtrlY;         },
                        26 => { return Command::CtrlZ;         },
                        27 => {
                            match non_blocking_read(trail) {
                                Ok(result) => {
                                    match result {
                                        Ok(read_count) => {
                                            if trail[0..2] == [91, 60] {
                                                if let Some(command) = parse_mouse_event(&trail[2..read_count]) {
                                                    return command;
                                                }
                                            }

                                            match read_count {
                                                2 => {
                                                    match trail[0] {
                                                        79 => {
                                                            match trail[1] {
                                                                80 => { return Command::F1; },
                                                                81 => { return Command::F2; },
                                                                82 => { return Command::F3; },
                                                                83 => { return Command::F4; },
                                                                _ => ()
                                                            }
                                                        },
                                                        91 => {
                                                            match trail[1] {
                                                                65 => { return Command::ArrowUp;    },
                                                                66 => { return Command::ArrowDown;  },
                                                                67 => { return Command::ArrowRight; },
                                                                68 => { return Command::ArrowLeft;  },
                                                                70 => { return Command::End;        },
                                                                72 => { return Command::Home;       },
                                                                90 => { return Command::ShiftTab;   },
                                                                _  => ()
                                                            }
                                                        },
                                                        _ => ()
                                                    }
                                                },
                                                3 => {
                                                    if trail[0] == 91 && trail[2] == 126 {
                                                        match trail[1] {
                                                            50 => { return Command::Insert;   },
                                                            51 => { return Command::Delete;   },
                                                            53 => { return Command::PageUp;   },
                                                            54 => { return Command::PageDown; },
                                                            _  => ()
                                                        }
                                                    }
                                                },
                                                4 => {
                                                    if trail[0] == 91 && trail[3] == 126 {
                                                        let n = (trail[1] - 48) * 10 + trail[2] - 48;
                                                        match n {
                                                            15 => { return Command::F5;  },
                                                            17 => { return Command::F6;  },
                                                            18 => { return Command::F7;  },
                                                            19 => { return Command::F8;  },
                                                            20 => { return Command::F9;  },
                                                            21 => { return Command::F10; },
                                                            23 => { return Command::F11; },
                                                            24 => { return Command::F12; },
                                                            _ => ()
                                                        }
                                                    }
                                                },
                                                5 => {
                                                    if trail[0..5] == [91, 51, 59, 53, 126] {
                                                        return Command::CtrlDelete;
                                                    }

                                                    if trail[0..3] == [91, 49, 59] {
                                                        match trail[3] {
                                                            50 => {
                                                                match trail[4] {
                                                                    65 => { return Command::ShiftArrowUp;    },
                                                                    66 => { return Command::ShiftArrowDown;  },
                                                                    67 => { return Command::ShiftArrowRight; },
                                                                    68 => { return Command::ShiftArrowLeft;  },
                                                                    70 => { return Command::ShiftEnd;        },
                                                                    72 => { return Command::ShiftHome;       },
                                                                    _  => ()
                                                                }
                                                            },
                                                            53 => {
                                                                match trail[4] {
                                                                    65 => { return Command::CtrlArrowUp;    },
                                                                    66 => { return Command::CtrlArrowDown;  },
                                                                    67 => { return Command::CtrlArrowRight; },
                                                                    68 => { return Command::CtrlArrowLeft;  },
                                                                    70 => { return Command::CtrlEnd;        },
                                                                    72 => { return Command::CtrlHome;       },
                                                                    _  => ()
                                                                }
                                                            },
                                                            _ => ()
                                                        }
                                                    }
                                                },
                                                9 => {
                                                    if trail[0..9] == [91, 50, 55, 59, 50, 59, 49, 51, 126] {
                                                        return Command::ShiftEnter;
                                                    }
                                                },
                                                _ => ()
                                            }
                                        },
                                        Err(_) => {
                                            return Command::Escape;
                                        }
                                    }
                                },
                                Err(err) => {
                                    return Command::Error(err);
                                }
                            }
                        },
                        127 => { return Command::Backspace; },
                        _   => ()
                    }
                },
                _ => ()
            }

            Command::Nop
        },
        Err(err) => {
            Command::Error(err)
        }
    }
}

fn non_blocking_read<const N: usize>(buffer: &mut [u8; N]) -> io::Result<io::Result<usize>> {
    let original_flags = unsafe { fcntl(STDIN_FILENO, F_GETFL) };
    if original_flags == -1 {
        return Err(io::Error::last_os_error());
    }

    let new_flags = original_flags | O_NONBLOCK;
    if unsafe { fcntl(STDOUT_FILENO, F_SETFL, new_flags) } == -1 {
        return Err(io::Error::last_os_error());
    }

    let result = stdin().read(buffer);

    if unsafe { fcntl(STDOUT_FILENO, F_SETFL, original_flags) } == -1 {
        return Err(io::Error::last_os_error());
    }

    Ok(result)
}

