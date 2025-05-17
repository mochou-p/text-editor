// text-editor/src/ansi/cursor.rs

#![allow(dead_code, reason = "for completeness")]

use super::CSI;


pub fn show() { print!("{CSI}?25h"); }
pub fn hide() { print!("{CSI}?25l"); }

pub fn move_up              (delta:  usize)             { print!("{CSI}{delta}A");        }
pub fn move_down            (delta:  usize)             { print!("{CSI}{delta}B");        }
pub fn move_right           (delta:  usize)             { print!("{CSI}{delta}C");        }
pub fn move_left            (delta:  usize)             { print!("{CSI}{delta}D");        }
pub fn move_to_next_line    (delta:  usize)             { print!("{CSI}{delta}E");        }
pub fn move_to_previous_line(delta:  usize)             { print!("{CSI}{delta}F");        }
pub fn move_to_x            (column: usize)             { print!("{CSI}{column}G");       }
pub fn move_to              (column: usize, row: usize) { print!("{CSI}{row};{column}H"); }

