// text-editor/src/colors.rs

use std::io::Write as _;

use betterm::{color, cursor, RESET_ALL};

use super::utf8::Utf8Len;
use super::Editor;


struct Word {
    start: usize,
    end:   usize
}

impl Word {
    const fn from(line_start: usize, start: usize, end: usize) -> Self {
        Self {
            start: line_start + start,
            end:   line_start + end
        }
    }
}

impl Editor {
    pub fn refresh(&mut self) {
        let words = self.parse_current_line_into_words();

        self.refresh_colors(words);
    }

    fn parse_current_line_into_words(&self) -> Vec<Word> {
        let mut words      = Vec::with_capacity(128);
        let     line       = &self.lines[self.cursor.y];
        let mut line_start = 0;

        loop {
            let slice = &line[line_start..];

            let Some(start) = slice.find(|ch: char| !ch.is_whitespace()) else {
                // NOTE: but why do i only get here once when spamming [Enter]?
                break;
            };

            let from_word_start = &slice[start..];
            let word_len_option = from_word_start.find(char::is_whitespace);

            if let Some(len) = word_len_option {
                let end  = start + len;
                let word = Word::from(line_start, start, end);

                words.push(word);
                line_start += end;
            } else {
                let end  = start + from_word_start.len();
                let word = Word::from(line_start, start, end);

                words.push(word);
                break;
            }
        }

        words
    }

    // TODO: make it context-aware and more sophisticated
    fn refresh_colors(&mut self, words: Vec<Word>) {
        for word in words {
            self.try_colorise_word(word);
        }

        write!(
            self.stdout,
            "{}{}",
            self.update_cursor_position(),
            RESET_ALL
        ).unwrap();

        self.stdout.flush().unwrap();
    }

    fn try_colorise_word(&mut self, word: Word) {
        let line = &self.lines[self.cursor.y];
        let text = &line[word.start..word.end];

        let color = get_text_color(text)
            .unwrap_or_else(|| RESET_ALL.to_string());

        write!(
            self.stdout,
            "{}{color}{text}",
            cursor::MoveToColumnAndRow(
                u16::try_from(
                    line[..word.start].utf8_len() + 1
                ).unwrap(),
                u16::try_from(self.cursor.y + 1 - self.scroll.y).unwrap()
            )
        ).unwrap();

        self.stdout.flush().unwrap();
    }
}

fn get_text_color(text: &str) -> Option<String> {
    match text {
        "macro_rules!" | "unsafe"
            => Some(color::FG_RED.to_string()),
        "bool" | "char" | "const" | "f32" | "f64" | "i8" | "i16" | "i32" | "i64" | "i128"
        | "isize" | "move" | "mut" | "ref" | "Self" | "static" | "str" | "String" | "u8"
        | "u16" | "u32" | "u64" | "u128" | "usize"
            => Some(color::FG_YELLOW.to_string()),
        "as" | "Err" | "false" | "None" | "Ok" | "Option" | "Result" | "self" | "Some" | "true"
            => Some(color::FG_CYAN.to_string()),
        "break" | "continue" | "crate" | "else" | "enum" | "extern" | "fn" | "for" | "if"
        | "impl" | "in" | "let" | "loop" | "match" | "mod" | "pub" | "return" | "struct"
        | "super" | "trait" | "type" | "use" | "where" | "while" | "async" | "await" | "dyn"
            => Some(color::FG_BLUE.to_string()),
        _
            => None
    }
}

