// mochou-p/text-editor/src/utils.rs

    mod to_with;
    mod utf8;
pub mod word;

pub use {to_with::ToWith, utf8::{Utf8, Utf8Mut}};


pub fn is_alphanumericx(ch: char) -> bool {
    ch == '_' || ch.is_alphanumeric()
}

