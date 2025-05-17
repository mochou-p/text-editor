// text-editor/src/ansi/clear.rs

#![allow(dead_code, reason = "for completeness")]

use super::CSI;


pub fn    under_cursor() { print!("{CSI}0J"); }
pub fn    above_cursor() { print!("{CSI}1J"); }
pub fn    whole_screen() { print!("{CSI}2J"); }

pub fn right_of_cursor() { print!("{CSI}0K"); }
pub fn  left_of_cursor() { print!("{CSI}1K"); }
pub fn    current_line() { print!("{CSI}2K"); }

