// mochou-p/text-editor/src/utils.rs

use crate::utf8::Utf8;


pub fn is_alphanumericx(ch: char) -> bool {
    ch == '_' || ch.is_alphanumeric()
}

pub fn find_to_left(string: &str, mut i: isize, f: impl Fn(char) -> bool) -> Option<isize> {
    let mut index = None;

    loop {
        i -= 1;

        if i == 0 {
            break;
        }

        if f(string.chars().nth(i as usize).unwrap()) {
            index = Some(i);
            break;
        }
    }

    index
}

pub fn find_to_right(string: &str, mut i: isize, f: impl Fn(char) -> bool) -> Option<isize> {
    let mut index = None;

    loop {
        i += 1;

        if i == string.utf8_len() {
            break;
        }

        if f(string.chars().nth(i as usize).unwrap()) {
            index = Some(i);
            break;
        }
    }

    index
}

