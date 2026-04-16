// mochou-p/text-editor/src/utils/word.rs

use super::Utf8;


pub fn to_left(string: &str, mut i: isize, f: impl Fn(char) -> bool) -> Option<isize> {
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

pub fn to_right(string: &str, mut i: isize, f: impl Fn(char) -> bool) -> Option<isize> {
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
