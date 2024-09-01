use std::collections::HashMap;
use std::iter::FromIterator;
use amina_core::cmd_manager::{ArgDescription, ArgType, ArgsList, CmdManager};
use amina_core::service::Service;

use crate::cli::InputHandler;

#[derive(PartialEq)]
enum ArgsParserState {
    WaitForArgNameStart,
    ReadingArgName,
    WaitForArgValue,
    ReadingStringValue,
    ReadingNonStringValue,
}

pub struct CmdManagerAdapter {
    cmd_manager: Service<CmdManager>,
}

impl CmdManagerAdapter {
    pub fn new(cmd_manager: Service<CmdManager>) -> Self {
        Self {
            cmd_manager
        }
    }
}

impl InputHandler for CmdManagerAdapter {
    fn handle(&self, input_line: &str) {
        let cmd_line = input_line.replace("\n", "");

        let args_start_option = cmd_line.find(" ");
        let (cmd_name, args_str) = match args_start_option {
            Some(args_start) => {
                (&cmd_line[..args_start], &cmd_line[(args_start + 1)..])
            },
            None => {
                (&cmd_line[..], "")
            },
        };

        log::debug!("CLI cmd: {:?}, args: {:?}", cmd_name, args_str);

        let cmd_list = self.cmd_manager.get_cmd_description().read().unwrap();
        match cmd_list.get(cmd_name) {
            Some(cmd_wrapper) => {
                let args = parse(args_str, &cmd_wrapper.description.args);
                if let Some(args) = args {
                    log::debug!("Cmd args: {:?}", &args);
                    (cmd_wrapper.handler)(&args);
                }
            },
            None => {
                log::error!("Unknown command '{}'", cmd_name);
            }
        }
    }
}

fn parse_raw(args_str: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();
    let mut state = ArgsParserState::WaitForArgNameStart;

    let mut name_vec = Vec::<char>::new();
    let mut value_vec = Vec::<char>::new();

    for c in args_str.chars() {
        match state {
            ArgsParserState::WaitForArgNameStart => {
                if c != ' ' {
                    name_vec.push(c);
                    state = ArgsParserState::ReadingArgName;
                }
            },
            ArgsParserState::ReadingArgName => {
                if c == ':' {
                    state = ArgsParserState::WaitForArgValue;
                } else {
                    name_vec.push(c);
                }
            },
            ArgsParserState::WaitForArgValue => {
                if c == '\'' {
                    state = ArgsParserState::ReadingStringValue;
                } else if c == ' ' {

                } else {
                    value_vec.push(c);
                    state = ArgsParserState::ReadingNonStringValue;
                }
            },
            ArgsParserState::ReadingStringValue => {
                if c == '\'' {
                    let name = String::from_iter(&name_vec);
                    let value = String::from_iter(&value_vec);
                    result.insert(name, value);
                    name_vec.clear();
                    value_vec.clear();
                    state = ArgsParserState::WaitForArgNameStart;
                } else {
                    value_vec.push(c);
                }
            },
            ArgsParserState::ReadingNonStringValue => {
                if c == ' ' {
                    let name = String::from_iter(&name_vec);
                    let value = String::from_iter(&value_vec);
                    result.insert(name, value);
                    name_vec.clear();
                    value_vec.clear();
                    state = ArgsParserState::WaitForArgNameStart;
                } else {
                    value_vec.push(c);
                }
            },
        }

    }

    if state == ArgsParserState::ReadingNonStringValue {
        let name = String::from_iter(&name_vec);
        let value = String::from_iter(&value_vec);
        result.insert(name, value);
        name_vec.clear();
        value_vec.clear();
    }

    return result;
}

fn parse(args_str: &str, args_description: &HashMap<String, ArgDescription>) -> Option<ArgsList> {
    let mut args_list = ArgsList::new();

    let raw_args = parse_raw(args_str);

    for (arg_name, description) in args_description {
        match raw_args.get(arg_name) {
            Some(arg_value_raw) => {
                match description.arg_type {
                    ArgType::U64 => {
                        match arg_value_raw.parse::<u64>() {
                            Ok(value) => args_list.put_u64(arg_name, value),
                            Err(_) => {
                                log::error!("Invalid int arg '{}': '{}'", arg_name, arg_value_raw);
                                return None;
                            }
                        }
                    },
                    ArgType::BOOL => {
                        if arg_value_raw.eq("y") {
                            args_list.put_bool(arg_name, true);
                        } else if arg_value_raw.eq("n") {
                            args_list.put_bool(arg_name, false);
                        } else {
                            log::error!("Invalid bool arg '{}', expected 'y' or 'n' but '{}' found", arg_name, arg_value_raw);
                            return None;
                        }
                    },
                    ArgType::STRING => {
                        args_list.put_string(arg_name, arg_value_raw.clone());
                    }
                }
            },
            None => {
                log::error!("Argument '{}' not found", arg_name);
                return None;
            }
        }
    }

    return Some(args_list);
}
