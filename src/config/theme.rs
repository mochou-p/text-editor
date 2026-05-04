// mochou-p/text-editor/src/config/theme.rs

use betterm::color::{BgRgb, FgRgb};


#[allow(dead_code)]
pub struct Theme {
    pub backgrounds: Backgrounds,
    pub foreground:  Foreground,
    pub special:     Special,
    pub ansi:        Ansi
}

pub struct Backgrounds {
    pub primary:   Background,
    pub secondary: Background
}

pub struct Background {
    pub active:   String,
    pub normal:   String,
    pub disabled: String
}

pub struct Foreground {
    pub active: String,
    pub normal: String
}

pub struct Special {
    pub error:    String,
    pub overflow: String
}

#[allow(dead_code)]
pub struct Ansi {
    pub red:     String,
    pub green:   String,
    pub yellow:  String,
    pub blue:    String,
    pub magenta: String,
    pub cyan:    String
}

impl Default for Theme {
    fn default() -> Self {
        Self::catppuccin_mocha()
    }
}

impl Theme {
    // https://catppuccin.com/palette/
    fn catppuccin_mocha() -> Self {
        Self {
            backgrounds: Backgrounds {
                primary: Background {
                    active:   BgRgb(49, 50, 68).to_string(),
                    normal:   BgRgb(30, 30, 46).to_string(),
                    disabled: BgRgb(17, 17, 27).to_string()
                },
                secondary: Background {
                    active:   BgRgb(69, 71, 90).to_string(),
                    normal:   BgRgb(49, 50, 68).to_string(),
                    disabled: BgRgb(24, 24, 37).to_string()
                }
            },
            foreground: Foreground {
                active: FgRgb(205, 214, 244).to_string(),
                normal: FgRgb(166, 173, 200).to_string()
            },
            special: Special {
                error:    BgRgb(210,  15, 57).to_string() + &FgRgb(17, 17, 27).to_string(),
                overflow: BgRgb(223, 142, 29).to_string() + &FgRgb(17, 17, 27).to_string()
            },
            ansi: Ansi {
                red:     FgRgb(243, 139, 168).to_string(),
                green:   FgRgb(166, 227, 161).to_string(),
                yellow:  FgRgb(249, 226, 175).to_string(),
                blue:    FgRgb(137, 180, 250).to_string(),
                magenta: FgRgb(245, 194, 231).to_string(),
                cyan:    FgRgb(137, 220, 235).to_string()
            }
        }
    }
}
