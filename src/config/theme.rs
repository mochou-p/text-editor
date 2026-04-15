// mochou-p/text-editor/src/config/theme.rs

use betterm::color;


pub struct Theme {
    pub void:      String,
    pub bg:        String,
    pub fg:        String,
    pub cursor_bg: String,
    pub cursor_fg: String,
    pub bad_trail: String
}

impl Default for Theme {
    fn default() -> Self {
        Self::catppuccin_mocha()
    }
}

impl Theme {
    fn catppuccin_mocha() -> Self {
        Self {
            void:      color::BgRgb( 17,  17,  27).to_string(),
            bg:        color::BgRgb( 30,  30,  46).to_string(),
            fg:        color::FgRgb(205, 214, 244).to_string(),
            cursor_bg: color::BgRgb( 49,  50,  68).to_string(),
            cursor_fg: color::FgRgb(137, 180, 250).to_string(),
            bad_trail: color::BgRgb(243, 139, 168).to_string()
        }
    }
}

