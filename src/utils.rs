// text-editor/src/utils.rs

pub type CastResult<RETURN, FROM, TO> = Result<RETURN, <TO as TryFrom<FROM>>::Error>;

#[macro_export]
macro_rules! error {
    ($str: expr) => {
        eprintln!("\x1b[31merror\x1b[0m: {}", $str);
    };

    ($fmt: expr, $($args: expr),+) => {
        eprint!("\x1b[31merror\x1b[0m: ");
        eprintln!($fmt $(,$args)+);
    };
}

#[macro_export]
macro_rules! warn {
    ($str: expr) => {
        eprintln!("\x1b[33mwarning\x1b[0m: {}", $str);
    };

    ($fmt: expr, $($args: expr),+) => {
        eprint!("\x1b[33mwarning\x1b[0m: ");
        eprintln!($fmt $(,$args)+);
    };
}

