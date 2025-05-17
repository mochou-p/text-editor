// text-editor/src/ansi/state.rs

use super::CSI;


pub fn alternative_screen() { print!("{CSI}?1049h"); }
pub fn      normal_screen() { print!("{CSI}?1049l"); }

