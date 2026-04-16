// mochou-p/text-editor/src/config/theme.rs

use betterm::color;


pub struct Theme {
        rainbow:   Option<usize>,
    pub void:      String,
    pub bg:        String,
    pub fg:        String,
    pub cursor_bg: String,
    pub cursor_fg: String,
    pub bad_trail: String,
    pub overflow:  String
}

impl Default for Theme {
    fn default() -> Self {
        Self::oklch_rainbow_black()
    }
}

static RAINBOW: &[&str] = &[
    "\x1b[38;2;236;139;170m",
    "\x1b[38;2;241;140;144m",
    "\x1b[38;2;240;144;118m",
    "\x1b[38;2;234;151;95m",
    "\x1b[38;2;221;160;78m",
    "\x1b[38;2;204;170;71m",
    "\x1b[38;2;181;179;79m",
    "\x1b[38;2;154;187;98m",
    "\x1b[38;2;124;194;123m",
    "\x1b[38;2;90;198;150m",
    "\x1b[38;2;56;199;176m",
    "\x1b[38;2;32;197;201m",
    "\x1b[38;2;49;192;223m",
    "\x1b[38;2;81;186;239m",
    "\x1b[38;2;114;178;249m",
    "\x1b[38;2;143;169;251m",
    "\x1b[38;2;170;161;247m",
    "\x1b[38;2;193;153;235m",
    "\x1b[38;2;212;146;217m",
    "\x1b[38;2;227;142;195m"
];

impl Theme {
    pub fn update(&mut self) {
        if let Some(i) = self.rainbow.as_mut() {
            self.cursor_fg = String::from(RAINBOW[*i]);
            *i = (*i + 1) % RAINBOW.len();
        }
    }
}

#[allow(dead_code)]
impl Theme {
    // https://catppuccin.com/palette/
    fn catppuccin_mocha() -> Self {
        Self {
            rainbow:   None,
            void:      color::BgRgb( 17,  17,  27).to_string(),
            bg:        color::BgRgb( 30,  30,  46).to_string(),
            fg:        color::FgRgb(205, 214, 244).to_string(),
            cursor_bg: color::BgRgb( 49,  50,  68).to_string(),
            cursor_fg: color::FgRgb(137, 220, 235).to_string(),
            bad_trail: color::BgRgb(243, 139, 168).to_string(),
            overflow:  format!(
                "{}{}",
                color::BgRgb(249, 226, 175).to_string(),
                color::FgRgb( 17,  17,  27).to_string()
            )
        }
    }

    // https://rosepinetheme.com/palette/
    fn rosepine_main() -> Self {
        Self {
            rainbow:   None,
            void:      color::BgRgb( 25,  23,  36).to_string(),
            bg:        color::BgRgb( 31,  29,  46).to_string(),
            fg:        color::FgRgb(224, 222, 244).to_string(),
            cursor_bg: color::BgRgb( 38,  35,  58).to_string(),
            cursor_fg: color::FgRgb(156, 207, 216).to_string(),
            bad_trail: color::BgRgb(235, 111, 146).to_string(),
            overflow:  format!(
                "{}{}",
                color::BgRgb(246, 193, 119).to_string(),
                color::FgRgb( 25,  23,  36).to_string()
            )
        }
    }

    // https://draculatheme.com/spec
    fn dracula_classic() -> Self {
        Self {
            rainbow:   None,
            void:      color::BgRgb( 33,  34,  44).to_string(),
            bg:        color::BgRgb( 40,  42,  54).to_string(),
            fg:        color::FgRgb(248, 248, 242).to_string(),
            cursor_bg: color::BgRgb( 68,  71,  90).to_string(),
            cursor_fg: color::FgRgb(139, 233, 253).to_string(),
            bad_trail: color::BgRgb(255,  85,  85).to_string(),
            overflow:  format!(
                "{}{}",
                color::BgRgb(241, 250, 140).to_string(),
                color::FgRgb( 33,  34,  44).to_string()
            )
        }
    }

    // https://github.com/tokyo-night/tokyo-night-vscode-theme?tab=readme-ov-file#color-palette
    fn tokyo_night() -> Self {
        Self {
            rainbow:   None,
            void:      color::BgRgb( 22,  22,  31).to_string(),
            bg:        color::BgRgb( 26,  27,  38).to_string(),
            fg:        color::FgRgb(172, 176, 208).to_string(),
            cursor_bg: color::BgRgb( 49,  55,  76).to_string(),
            cursor_fg: color::FgRgb(125, 207, 255).to_string(),
            bad_trail: color::BgRgb(247, 118, 142).to_string(),
            overflow:  format!(
                "{}{}",
                color::BgRgb(224, 175, 104).to_string(),
                color::FgRgb( 22,  22,  31).to_string()
            )
        }
    }

    fn oklch_rainbow_black() -> Self {
        Self {
            rainbow:   Some(0),
            void:      color::BgRgb(  0,   0,   0).to_string(),
            bg:        color::BgRgb( 15,  15,  15).to_string(),
            fg:        color::FgRgb(155, 155, 155).to_string(),
            cursor_bg: color::BgRgb( 25,  25,  25).to_string(),
            cursor_fg: color::FgRgb(255, 255, 255).to_string(),
            bad_trail: color::BgRgb(255,   0,   0).to_string(),
            overflow:  format!(
                "{}{}",
                color::BgRgb(255, 255,   0).to_string(),
                color::FgRgb(  5,   5,   5).to_string()
            )
        }
    }
}
