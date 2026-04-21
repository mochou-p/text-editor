// mochou-p/text-editor/src/config/theme.rs

use betterm::color::{BgRgb, FgRgb};


pub struct Theme {
    pub backgrounds: Backgrounds,
    pub foreground:  Foreground,
    pub special:     Special
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

impl Default for Theme {
    fn default() -> Self {
        Self::catppuccin_mocha()
    }
}

#[allow(dead_code)]
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
            }
        }
    }
}
