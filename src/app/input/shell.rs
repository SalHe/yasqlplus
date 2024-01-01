use std::{cell::RefCell, rc::Rc, sync::RwLock};

use rustyline::{
    history::FileHistory, Cmd, CompletionType, Config, EditMode, Editor, EventHandler, KeyEvent,
};

use crate::command::{parse_command, Command, ParseError};

use crate::app::{context::Context, helper::YspHelper};

use super::{Input, InputError, InputSettings};

const HISTORY_FILE: &str = "yasqlplus-history.txt";

pub struct ShellInput {
    context: Rc<RwLock<Context>>,
    rl: RefCell<Editor<YspHelper, FileHistory>>,
}

impl ShellInput {
    pub fn new(context: Rc<RwLock<Context>>) -> Result<Self, InputError> {
        let config = Config::builder()
            .history_ignore_space(true)
            .completion_type(CompletionType::Circular)
            .edit_mode(EditMode::Vi)
            .auto_add_history(true)
            .history_ignore_dups(true)?
            .build();
        let mut rl = Editor::with_config(config)?;
        rl.set_helper(Some(YspHelper::new(context.clone())));
        rl.bind_sequence(KeyEvent::alt('s'), EventHandler::Simple(Cmd::Newline));
        let _ = rl.load_history(HISTORY_FILE);
        Ok(Self {
            rl: RefCell::new(rl),
            context,
        })
    }
}

impl InputSettings for ShellInput {
    fn need_echo(&self) -> bool {
        false
    }
}

impl Input for ShellInput {
    fn get_command(&self) -> Result<Option<(Command, String)>, InputError> {
        let input = self
            .rl
            .borrow_mut()
            .readline(&self.context.read().unwrap().get_prompt())?;
        let command = match parse_command(&input) {
            Ok(command) => Some((command, input)),
            Err(ParseError::Empty) => None,
            Err(err) => return Err(err.into()),
        };

        Ok(command)
    }

    fn line(&self, prompt: &str) -> Result<String, InputError> {
        let mut rl = self.rl.borrow_mut();
        rl.helper_mut().unwrap().disable_validation();
        let input = rl.readline(prompt);
        rl.helper_mut().unwrap().enable_validation();
        input.map_err(InputError::from)
    }
}

impl Drop for ShellInput {
    fn drop(&mut self) {
        let _ = self.rl.borrow_mut().append_history(HISTORY_FILE);
    }
}
