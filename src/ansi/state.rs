// text-editor/src/ansi/state.rs

use super::CSI;


pub fn alternative_screen() { print!("{CSI}?1049h"); }
pub fn      normal_screen() { print!("{CSI}?1049l"); }

pub fn  enable_mouse() { print!("{CSI}?1006h{CSI}?1003h"); }
pub fn disable_mouse() { print!("{CSI}?1003l{CSI}?1006l"); }

