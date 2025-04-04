// text-editor/src/config.rs

use std::{
    fs::{File, read_to_string},
    io::{self, ErrorKind, Write as _}
};

use super::editor::TextEditor;


macro_rules! warn {
    ($str: expr) => {
        eprintln!("\x1b[33mwarning\x1b[0m: {}", $str);
    };

    ($fmt: expr, $($args: expr),+) => {
        eprint!("\x1b[33mwarning\x1b[0m: ");
        eprintln!($fmt $(,$args)+);
    };
}

static CONFIG_FILE:    &str = "config.conf";
static CORRECT_FORMAT: &str = "         correct format: `property = value`";

#[derive(Default)]
pub struct Config {
    pub halignment: HAlignment,
    pub valignment: VAlignment
}

#[derive(Default)]
struct ConfigOptions {
    halignment: Option<HAlignment>,
    valignment: Option<VAlignment>
}

impl Config {
    pub fn load() -> io::Result<Self> {
        let config = match read_to_string(CONFIG_FILE) {
            Ok(string) => {
                Self::parse_conf(&string)
            },
            Err(err) => {
                if err.kind() != ErrorKind::NotFound {
                    todo!();
                }

                warn!("configuration file not found, creating `{}` with default values", CONFIG_FILE);

                let mut file = File::create(CONFIG_FILE)?;
                file.write_all(b"alignment-horizontal = left    # left/center-left/center/center-right/right\nalignment-vertical   = center  # top/center/bottom\n\n")?;

                Self::default()
            }
        };

        Ok(config)
    }

    fn parse_conf(string: &str) -> Self {
        let mut config = ConfigOptions::default();

        for (i, mut line) in string.lines().enumerate() {
            if line.is_empty() {
                continue;
            }

            if let Some(comment) = line.find('#') {
                if comment == 0 {
                    continue;
                }

                if line.chars().nth(comment - 1).unwrap() != ' ' {
                    warn!("{CONFIG_FILE}:{}:{}: comments in `.conf` files need a space before `#`", i + 1, comment + 1);
                    continue;
                }

                line = &line[..comment];
            }

            let mut leading = line.len();
            line = line.trim_start();
            leading -= line.len();
            line = line.trim_end();

            if line.is_empty() {
                continue;
            }

            if let Some(equals) = line.find('=') {
                let len = line.len();
                if equals == len - 1 {
                    warn!("{CONFIG_FILE}:{}:{}: `=` needs a value after it\n{CORRECT_FORMAT}", i + 1, len + leading);
                    continue;
                }

                if line[equals + 1..].find('=').is_some() {
                    warn!("{CONFIG_FILE}:{}: multiple `=`s are not valid `.conf`\n{CORRECT_FORMAT}", i + 1);
                    continue;
                }

                if equals == 0 {
                    warn!("{CONFIG_FILE}:{}:{}: `=` needs a property in front of it\n{CORRECT_FORMAT}", i + 1, 1 + leading);
                    continue;
                }

                let property = line[..equals    ].trim_end();
                let value    = line[equals + 1..].trim_start();

                // TODO: clean up
                match property {
                    "alignment-horizontal" => {
                        if config.halignment.is_some() {
                            warn!("{CONFIG_FILE}:{}: duplicate declaration of `alignment-horizontal`", i + 1);
                            continue;
                        }

                        match value {
                            "left"         => { config.halignment = Some(HAlignment::Left)        },
                            "center-left"  => { config.halignment = Some(HAlignment::CenterLeft)  },
                            "center"       => { config.halignment = Some(HAlignment::Center)      },
                            "center-right" => { config.halignment = Some(HAlignment::CenterRight) },
                            "right"        => { config.halignment = Some(HAlignment::Right)       },
                            other => {
                                warn!("{CONFIG_FILE}:{}: `{other}` is not a valid `alignment-horizontal` value\n         values: `left`, `center-left`, `center`, `center-right`, `right`", i + 1);
                                continue;
                            }
                        }
                    },
                    "alignment-vertical" => {
                        if config.valignment.is_some() {
                            warn!("{CONFIG_FILE}:{}: duplicate declaration of `alignment-vertical`", i + 1);
                            continue;
                        }

                        match value {
                            "top"    => { config.valignment = Some(VAlignment::Top)    },
                            "center" => { config.valignment = Some(VAlignment::Center) },
                            "bottom" => { config.valignment = Some(VAlignment::Bottom) },
                            other => {
                                warn!("{CONFIG_FILE}:{}: `{other}` is not a valid `alignment-vertical` value\n         values: `top`, `center`, `bottom`", i + 1);
                                continue;
                            }
                        }
                    },
                    other => {
                        warn!("{CONFIG_FILE}:{}: `{other}` is not a valid property\n         properties: `alignment-horizontal`, `alignment-vertical`", i + 1);
                        continue;
                    }
                }
            } else {
                warn!("{CONFIG_FILE}:{}: not a valid assignment\n{CORRECT_FORMAT}", i + 1);
                continue;
            }
        }

        Self {
            halignment: config.halignment.unwrap_or_default(),
            valignment: config.valignment.unwrap_or_default()
        }
    }
}

#[derive(Default, PartialEq, Eq)]
pub enum HAlignment {
    #[default]
    Left,
    CenterLeft,
    Center,
    CenterRight,
    Right
}

impl HAlignment {
    pub const fn needs_longest_line(&self) -> bool {
        matches!(self, Self::CenterLeft | Self::CenterRight)
    }

    pub const fn get_starting_x(&self, columns: u16) -> u16 {
        match self {
            Self::Left  => 0,
            Self::Right => columns - 1,
            _           => columns / 2 - 1
        }
    }

    pub fn get_x(&self, te: &TextEditor) -> Result<u16, <u16 as TryFrom<usize>>::Error> {
        Ok(match self {
            Self::Left => 0,
            Self::CenterLeft => {
                te.columns
                    / 2
                - 1
                - te.longest_line.length
                    / 2
            },
            Self::Center => {
                te.columns
                    / 2
                - 1
                - u16::try_from(te.lines[te.cursor_y as usize].len())?
                    / 2
            },
            Self::CenterRight => {
                te.columns
                    / 2
                - 1
                + te.longest_line.length
                    / 2
                - u16::try_from(te.lines[te.cursor_y as usize].len())?
            },
            Self::Right => {
                te.columns
                - 1
                - u16::try_from(te.lines[te.cursor_y as usize].len())?
            }
        })
    }
}

#[derive(Default)]
pub enum VAlignment {
    #[default]
    Top,
    Center,
    Bottom
}

impl VAlignment {
    pub fn get_y(&self, te: &TextEditor) -> Result<u16, <u16 as TryFrom<usize>>::Error> {
        Ok(match self {
            Self::Top    => te.cursor_y,
            Self::Center => {
                te.cursor_y
                + te.rows
                    / 2
                - u16::try_from(te.lines.len())?
                    / 2
                - 1
            },
            Self::Bottom => {
                te.rows
                - 1
                - u16::try_from(te.lines.len())?
                + te.cursor_y
            }
        })
    }
}

