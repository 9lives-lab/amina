use std::collections::HashMap;
use std::iter::FromIterator;

use crate::cmd_manager::{ArgDescription, ArgsList, ArgType, CmdDescription, CmdManager};

#[derive(PartialEq)]
enum ArgsParserState {
    WaitForArgNameStart,
    ReadingArgName,
    WaitForArgValue,
    ReadingStringValue,
    ReadingNonStringValue,
}

pub struct CmdManagerCliAdapter<'a> {
    cmd_manager: &'a CmdManager,
}

impl <'a> CmdManagerCliAdapter<'a> {

    pub fn new(cmd_manager: &'a CmdManager) -> Self {
        Self {
            cmd_manager
        }
    }

    pub fn run(&self) {
        self.cmd_manager.add_command(CmdDescription {
                call_name: "q".to_string(),
                description: Some("Exit the application".to_string()),
                args: HashMap::new(),
            },
            |_| {}
        );

        loop {
            let mut cmd_line = String::new();
            std::io::stdin().read_line(&mut cmd_line).unwrap();

            let cmd_line = cmd_line.replace("\n", "");

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
            let cmd_wrapper = cmd_list.get(cmd_name).unwrap();

            let args = parse(args_str, &cmd_wrapper.description.args);
            log::debug!("Cmd args: {:?}", &args);

            if cmd_name.eq("q") {
                break;
            } else {
                (cmd_wrapper.handler)(&args);
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

fn parse(args_str: &str, args_description: &HashMap<String, ArgDescription>) -> ArgsList {
    let mut args_list = ArgsList::new();

    let raw_args = parse_raw(args_str);

    for (arg_name, description) in args_description {
        match raw_args.get(arg_name) {
            Some(arg_value_raw) => {
                match description.arg_type {
                    ArgType::U64 => {
                        let value: u64 = arg_value_raw.parse::<u64>().unwrap();
                        args_list.put_u64(arg_name, value);
                    },
                    ArgType::BOOL => {
                        if arg_value_raw.eq("y") {
                            args_list.put_bool(arg_name, true);
                        } else if arg_value_raw.eq("n") {
                            args_list.put_bool(arg_name, false);
                        } else {
                            unreachable!()
                        }
                    },
                    ArgType::STRING => {
                        args_list.put_string(arg_name, arg_value_raw.clone());
                    }
                }
            },
            None => {
                unreachable!()
            }
        }
    }

    return args_list;
}
