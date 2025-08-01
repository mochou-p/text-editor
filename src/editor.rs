// text-editor/src/editor.rs

use libc::{
    self,
    fcntl, iscntrl, tcgetattr, tcsetattr,
    BRKINT, CS8, ECHO, F_GETFL, F_SETFL, ICANON, ICRNL, IEXTEN, INPCK, ISIG, ISTRIP, IXON, O_NONBLOCK, OPOST, STDIN_FILENO, STDOUT_FILENO, TCSAFLUSH
};

use std::{
    io::{self, stdin, stdout, Read as _, Write as _},
    mem::zeroed
};

use crate::ansi::{cursor, state, clear};


pub struct Editor {
}

impl Editor {
    pub fn run() -> io::Result<()> {
        let original_termios = get_termios()?;
        let      new_termios = prepare_termios(original_termios);

        let mut stdin  = stdin();
        let mut stdout = stdout();

        set_termios(new_termios)?;
        state::alternative_screen();
        clear::whole_screen();
        cursor::move_to(1, 1);
        let _ = stdout.flush();

        let mut buffer = [0u8; 1];
        let mut error  = None;

        loop {
            match stdin.read_exact(&mut buffer) {
                Ok(()) => {
                    if buffer[0] == 3 {
                        break;
                    }

                    print!("{buffer:?} ; {}", unsafe { iscntrl(i32::from(buffer[0])) });
                    cursor::move_to_next_line(1);
                    let _ = stdout.flush();
                },
                Err(err) => {
                    error = Some(err);
                }
            }
        }

        state::normal_screen();
        set_termios(original_termios)?;

        if let Some(err) = error {
            eprintln!("\x1b[31merror:\x1b[0m {err}");
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

const fn prepare_termios(mut termios: libc::termios) -> libc::termios {
    termios.c_iflag &= !(BRKINT | ICRNL | INPCK | ISTRIP | IXON);
    termios.c_oflag &= !(OPOST);
    termios.c_cflag |=   CS8;
    termios.c_lflag &= !(ECHO | ICANON | IEXTEN | ISIG);

    termios
}

fn set_termios(termios: libc::termios) -> io::Result<()> {
    if unsafe { tcsetattr(STDIN_FILENO, TCSAFLUSH, &raw const termios) } != 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}

fn _non_blocking_read<const N: usize>(buf: &mut [u8; N]) -> io::Result<bool> {
    let flags = unsafe { fcntl(STDIN_FILENO, F_GETFL) };
    if flags == -1 {
        return Err(io::Error::last_os_error());
    }

    let new_flags = flags | O_NONBLOCK;
    if unsafe { fcntl(STDOUT_FILENO, F_SETFL, new_flags) } == -1 {
        return Err(io::Error::last_os_error());
    }

    let ret = stdin().read(buf).is_ok();

    if unsafe { fcntl(STDOUT_FILENO, F_SETFL, flags) } == -1 {
        return Err(io::Error::last_os_error());
    }

    Ok(ret)
}

