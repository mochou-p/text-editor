// text-editor/src/config.rs

use std::{
    fs::{self, File},
    io::{self, ErrorKind, Write as _}
};

use {
    super::{editor::TextEditor, utils::CastResult},
    crate::{error, warn}
};


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
        let config = match fs::read_to_string(CONFIG_FILE) {
            Ok(string) => {
                Self::parse_conf(&string)
            },
            Err(err) => {
                match err.kind() {
                    ErrorKind::NotFound => (),
                    ErrorKind::PermissionDenied => {
                        error!("current user lacks read privilege to configuration `{}`", CONFIG_FILE);

                        return Err(err);
                    },
                    _ => {
                        error!("std::fs::read_to_string failed to read configuration `{}`", CONFIG_FILE);

                        return Err(err);
                    }
                }

                warn!("configuration file not found, creating `{}` with default values", CONFIG_FILE);

                let mut file = File::create(CONFIG_FILE)?;
                file.write_all(b"alignment-horizontal = left  # left/center-left/center/center-right/right\nalignment-vertical   = top   # top/center/bottom\n\n")?;

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

    pub fn get_starting_x(&self, te: &TextEditor) -> CastResult<u16, usize, u16> {
        Ok(if *self == Self::Right {
            te.columns - 1
        } else {
            let len = u16::try_from(te.lines[te.lines.len() - 1].len())?;

            match self {
                Self::Left       => len,
                Self::CenterLeft => {
                    (te.columns - te.longest_line.length) / 2 + len
                },
                Self::Center => {
                    (te.columns + len) / 2
                    - (1 - (len % 2))
                },
                Self::CenterRight => {
                    (te.columns + te.longest_line.length) / 2 - 1
                },
                Self::Right => unreachable!()
            }
        })
    }

    pub fn get_x(&self, te: &TextEditor) -> CastResult<u16, usize, u16> {
        Ok(match self {
            Self::Left => 0,
            Self::CenterLeft => {
                te.columns / 2 - 1
                - te.longest_line.length / 2
            },
            Self::Center => {
                te.columns / 2 - 1
                - u16::try_from(te.lines[te.cursor_y as usize].len())? / 2
            },
            Self::CenterRight => {
                te.columns / 2 - 1
                + te.longest_line.length / 2
                - u16::try_from(te.lines[te.cursor_y as usize].len())?
            },
            Self::Right => {
                te.columns - 1
                - u16::try_from(te.lines[te.cursor_y as usize].len())?
            }
        })
    }
}

#[derive(Default, PartialEq, Eq)]
pub enum VAlignment {
    #[default]
    Top,
    Center,
    Bottom
}

impl VAlignment {
    pub fn get_y(&self, te: &TextEditor) -> CastResult<u16, usize, u16> {
        Ok(match self {
            Self::Top    => te.cursor_y,
            Self::Center => {
                te.cursor_y
                + te.rows / 2 - 1
                - u16::try_from(te.lines.len())? / 2
            },
            Self::Bottom => {
                te.rows - 1
                - u16::try_from(te.lines.len())?
                + te.cursor_y
            }
        })
    }
}

