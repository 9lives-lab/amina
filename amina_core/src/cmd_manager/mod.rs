pub mod cli_adapter;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use serde::{Serialize, Deserialize};

use crate::rpc::{EmptyData, Rpc};
use crate::service::{Context, ServiceApi, ServiceInitializer};

#[derive(Serialize, Clone, Debug)]
pub enum ArgType {
    U64,
    BOOL,
    STRING,
}

#[derive(Serialize, Clone, Debug)]
pub struct ArgDescription {
    pub call_name: String,
    pub description: Option<String>,
    pub arg_type: ArgType,
}

#[derive(Serialize, Clone, Debug)]
pub struct CmdDescription {
    pub call_name: String,
    pub description: Option<String>,
    pub args: HashMap<String, ArgDescription>,
}

pub struct CmdBuilder {
    description: CmdDescription,
}

impl CmdBuilder {

    pub fn new(call_name: &str) -> Self {
        Self {
            description: CmdDescription {
                call_name: call_name.to_string(),
                description: None,
                args: HashMap::new(),
            }
        }
    }

    pub fn add_description(mut self, description: &str) -> Self {
        self.description.description = Some(description.to_string());
        self
    }

    pub fn add_arg(mut self, arg: ArgDescription) -> Self {
        self.description.args.insert(arg.call_name.clone(), arg);
        self
    }

    pub fn build(self) -> CmdDescription {
        self.description
    }

}

pub struct ArgBuilder {
    description: ArgDescription,
}

impl ArgBuilder {

    pub fn new(call_name: &str, arg_type: ArgType) -> Self {
        Self {
            description: ArgDescription {
                call_name: call_name.to_string(),
                description: None,
                arg_type,
            }
        }
    }

    pub fn add_description(mut self, description: &str) -> Self {
        self.description.description = Some(description.to_string());
        self
    }

    pub fn build(self) -> ArgDescription {
        self.description
    }

}

#[derive(Deserialize, Debug)]
pub struct ArgsList {
    u64_list: HashMap<String, u64>,
    bool_list: HashMap<String, bool>,
    string_list: HashMap<String, String>,
}

impl ArgsList {

    pub fn new() -> Self {
        Self {
            u64_list: HashMap::new(),
            bool_list: HashMap::new(),
            string_list: HashMap::new(),
        }
    }

    pub fn get_u64(&self, arg_call_name: &str) -> u64 {
        *self.u64_list.get(arg_call_name).unwrap()
    }

    pub fn put_u64(&mut self, arg_call_name: &str, value: u64) {
        self.u64_list.insert(arg_call_name.to_string(), value);
    }

    pub fn get_bool(&self, arg_call_name: &str) -> bool {
        *self.bool_list.get(arg_call_name).unwrap()
    }

    pub fn put_bool(&mut self, arg_call_name: &str, value: bool) {
        self.bool_list.insert(arg_call_name.to_string(), value);
    }

    pub fn get_string(&self, arg_call_name: &str) -> String {
        self.string_list.get(arg_call_name).unwrap().to_string()
    }

    pub fn put_string(&mut self, arg_call_name: &str, value: String) {
        self.string_list.insert(arg_call_name.to_string(), value);
    }

}

pub struct CmdWrapper {
    description: CmdDescription,
    handler: Box<dyn Fn(&ArgsList) + Sync + Send + 'static>,
}

#[derive(Serialize)]
pub struct CommandsDescription {
    pub command_names: Vec<String>,
}

pub struct CmdManager {
    cmd_map: RwLock<HashMap<String, CmdWrapper>>,
}

impl CmdManager {

    pub fn new() -> Self {
        let cmd_map = HashMap::new();

        Self {
            cmd_map: RwLock::new(cmd_map),
        }
    }

    pub fn add_command<F>(&self, description: CmdDescription, handler: F) where
        F: Fn(&ArgsList) + Send + Sync + 'static
    {
        let mut cmd_map = self.cmd_map.write().unwrap();
        cmd_map.insert(description.call_name.clone(), CmdWrapper {
            description,
            handler: Box::new(handler),
        });
    }

    pub fn get_cmd_description(&self) -> &RwLock<HashMap<String, CmdWrapper>> {
        &self.cmd_map
    }

    pub fn handle(&self, cmd_call_name: &str, args: &ArgsList) {
        let cmd_map = self.cmd_map.read().unwrap();
        let handler = &cmd_map.get(cmd_call_name).unwrap().handler;
        handler(args);
    }

    pub fn get_commands_description(&self) -> CommandsDescription {
        let mut command_names = Vec::new();

        let cmd_map = self.cmd_map.read().unwrap();
        for (cmd_name, _) in cmd_map.iter() {
            command_names.push(cmd_name.to_string());
        }
        
        CommandsDescription {
            command_names
        }
    }

    pub fn get_command_description(&self, cmd_name: &str) -> CmdDescription {
        let cmd_map = self.cmd_map.read().unwrap();
        let cmd_wrapper = cmd_map.get(cmd_name).unwrap();
        return cmd_wrapper.description.clone();
    }

}

impl ServiceApi for CmdManager {

}

impl ServiceInitializer for CmdManager {
    fn initialize(context: &Context) -> Arc<Self> {
        let rpc = context.get_service::<Rpc>();
        let cmd_manager = Arc::new(Self::new());

        #[derive(Deserialize)]
        struct HandleCmdReq {
            cmd_name: String,
            args: ArgsList,
        }
        let cmd_manager_copy = cmd_manager.clone();
        rpc.on_generic_call_fn("amina.cmd_manager.handle", move |args: &HandleCmdReq| {
            cmd_manager_copy.handle(args.cmd_name.as_str(), &args.args);
        });

        let cmd_manager_copy = cmd_manager.clone();
        rpc.on_generic_call_fn("amina.cmd_manager.get_commands_description", move |_: &EmptyData| {
            return cmd_manager_copy.get_commands_description();
        });

        #[derive(Deserialize)]
        struct GetCommandDescriptionReq {
            cmd_name: String,
        }
        let cmd_manager_copy = cmd_manager.clone();
        rpc.on_generic_call_fn("amina.cmd_manager.get_command_description", move |req: &GetCommandDescriptionReq| {
            return cmd_manager_copy.get_command_description(req.cmd_name.as_str());
        });

        return cmd_manager;
    }
}
