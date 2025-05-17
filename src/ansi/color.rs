// text-editor/src/ansi/color.rs

#![allow(dead_code, reason = "for completeness")]

use super::CSI;


// these are used after CSI
pub static RESET:                     &str = "0m";

pub static FOREGROUND_BLACK:          &str = "30m";
pub static FOREGROUND_RED:            &str = "31m";
pub static FOREGROUND_GREEN:          &str = "32m";
pub static FOREGROUND_YELLOW:         &str = "33m";
pub static FOREGROUND_BLUE:           &str = "34m";
pub static FOREGROUND_MAGENTA:        &str = "35m";
pub static FOREGROUND_CYAN:           &str = "36m";
pub static FOREGROUND_WHITE:          &str = "37m";

pub static BACKGROUND_BLACK:          &str = "40m";
pub static BACKGROUND_RED:            &str = "41m";
pub static BACKGROUND_GREEN:          &str = "42m";
pub static BACKGROUND_YELLOW:         &str = "43m";
pub static BACKGROUND_BLUE:           &str = "44m";
pub static BACKGROUND_MAGENTA:        &str = "45m";
pub static BACKGROUND_CYAN:           &str = "46m";
pub static BACKGROUND_WHITE:          &str = "47m";

pub static BRIGHT_FOREGROUND_BLACK:   &str = "90m";
pub static BRIGHT_FOREGROUND_RED:     &str = "91m";
pub static BRIGHT_FOREGROUND_GREEN:   &str = "92m";
pub static BRIGHT_FOREGROUND_YELLOW:  &str = "93m";
pub static BRIGHT_FOREGROUND_BLUE:    &str = "94m";
pub static BRIGHT_FOREGROUND_MAGENTA: &str = "95m";
pub static BRIGHT_FOREGROUND_CYAN:    &str = "96m";
pub static BRIGHT_FOREGROUND_WHITE:   &str = "97m";

pub static BRIGHT_BACKGROUND_BLACK:   &str = "100m";
pub static BRIGHT_BACKGROUND_RED:     &str = "101m";
pub static BRIGHT_BACKGROUND_GREEN:   &str = "102m";
pub static BRIGHT_BACKGROUND_YELLOW:  &str = "103m";
pub static BRIGHT_BACKGROUND_BLUE:    &str = "104m";
pub static BRIGHT_BACKGROUND_MAGENTA: &str = "105m";
pub static BRIGHT_BACKGROUND_CYAN:    &str = "106m";
pub static BRIGHT_BACKGROUND_WHITE:   &str = "107m";

static LUT_FOREGROUND: &str = "38;5;";
static LUT_BACKGROUND: &str = "48;5;";

static RGB_FOREGROUND: &str = "38;2;";
static RGB_BACKGROUND: &str = "48;2;";

pub fn foreground_256(n: u8)               { print!("{CSI}{LUT_FOREGROUND}{n}m"); }
pub fn background_256(n: u8)               { print!("{CSI}{LUT_BACKGROUND}{n}m"); }

pub fn foreground_rgb(r: u8, g: u8, b: u8) { print!("{CSI}{RGB_FOREGROUND}{r};{g};{b}m"); }
pub fn background_rgb(r: u8, g: u8, b: u8) { print!("{CSI}{RGB_BACKGROUND}{r};{g};{b}m"); }

