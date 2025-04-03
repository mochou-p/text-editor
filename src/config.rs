// text-editor/src/config.rs

use std::{
    fs::{File, read_to_string},
    io::{self, ErrorKind, Write as _}
};


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
    halignment: Alignment
}

#[derive(Default)]
struct ConfigOptions {
    halignment: Option<Alignment>
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
                file.write_all(b"alignment = center-left  # left/center-left/center/center-right/right\n")?;

                Self::default()
            }
        };

        if config.halignment != Alignment::default() {
            warn!("only `center-left` is currently implemented for `alignment`");
        }

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

                match property {
                    "alignment" => {
                        if config.halignment.is_some() {
                            warn!("{CONFIG_FILE}:{}: duplicate declaration of `alignment`", i + 1);
                            continue;
                        }

                        match value {
                            "left"         => { config.halignment = Some(Alignment::Left)        },
                            "center-left"  => { config.halignment = Some(Alignment::CenterLeft)  },
                            "center"       => { config.halignment = Some(Alignment::Center)      },
                            "center-right" => { config.halignment = Some(Alignment::CenterRight) },
                            "right"        => { config.halignment = Some(Alignment::Right)       },
                            other => {
                                warn!("{CONFIG_FILE}:{}: `{other}` is not a valid `alignment` value\n         values: `left`, `center-left`, `center`, `center-right`, `right`", i + 1);
                                continue;
                            }
                        }
                    },
                    other => {
                        warn!("{CONFIG_FILE}:{}: `{other}` is not a valid property\n         properties: `alignment`", i + 1);
                        continue;
                    }
                }
            } else {
                warn!("{CONFIG_FILE}:{}: not a valid assignment\n{CORRECT_FORMAT}", i + 1);
                continue;
            }
        }

        Self { halignment: config.halignment.unwrap_or_default() }
    }
}

#[derive(Default, PartialEq)]
pub enum Alignment {
    Left,
    #[default]
    CenterLeft,
    Center,
    CenterRight,
    Right
}

