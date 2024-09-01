pub mod adapters;

use std::{io::Write, path::Path};
use log::LevelFilter;
use env_logger::Builder;
use chrono::Local;
use liner::{Completer, Context, Prompt};

struct EmptyCompleter;

impl Completer for EmptyCompleter {
    fn completions(&mut self, _start: &str) -> Vec<String> {
        Vec::new()
    }
}

pub trait InputHandler {
    fn handle(&self, input_line: &str);
}

pub struct CliContext {
    liner_ctx: Context,
    input_handler: Box<dyn InputHandler>,
}

impl CliContext {
    pub fn create(input_handler: Box<dyn InputHandler>, filters: Vec<(String, log::LevelFilter)>, history_file: &Path) -> Self {
        let mut builder = Builder::from_default_env();

        builder.format(|buf, record| {
                write!(buf, "[{}][{}][{}] {}\r\n", Local::now().format("%Y-%m-%d %H:%M:%S"), record.level(), record.target(), record.args())
        });
        builder.filter(None, LevelFilter::Debug);

        for (module, level) in filters {
            builder.filter(Some(&module), level);
        }

        builder.init();

        let mut liner_ctx = Context::new();

        if let Err(err) = liner_ctx.history.set_file_name_and_load_history(history_file) {
            log::error!("Error loading commands history: {}", err);
        }

        Self {
            liner_ctx,
            input_handler
        }
    }

    pub fn run(&mut self) {
        loop {
            let cmd_line = self.liner_ctx.read_line(Prompt::from(">"), None, &mut EmptyCompleter);
            let cmd_line = match cmd_line {
                Ok(cmd_line) => cmd_line,
                Err(_) => break,
            };

            if cmd_line.is_empty() {
                continue;
            }

            if let Err(err) = self.liner_ctx.history.push(cmd_line.clone().into()) {
                log::error!("Error pushing command line to history: {}", err);
            }
            self.liner_ctx.history.commit_to_file();

            self.input_handler.handle(&cmd_line);
        }
    }
}

