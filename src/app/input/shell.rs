use std::sync::Arc;
use std::{cell::RefCell, sync::RwLock};

use colored::Colorize;
use rustyline::{
    history::FileHistory, Cmd, CompletionType, Config, EditMode, Editor, EventHandler, KeyEvent,
};
use rustyline::{DefaultEditor, KeyCode, Modifiers};

use crate::command::{parse_command, Command, ParseError};

use crate::app::{context::Context, helper::YspHelper};

use super::{Input, InputError, INDICATOR};

pub struct ShellInput {
    context: Arc<RwLock<Context>>,
    rl: RefCell<Editor<YspHelper, FileHistory>>,
    rl2: RefCell<DefaultEditor>,
    history_file: String,
}

impl ShellInput {
    pub fn new(context: Arc<RwLock<Context>>, history_file: String) -> Result<Self, InputError> {
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
        rl.bind_sequence(
            KeyEvent(KeyCode::BracketedPasteStart, Modifiers::NONE),
            Cmd::Noop,
        );
        let _ = rl.load_history(&history_file);
        Ok(Self {
            rl: RefCell::new(rl),
            rl2: RefCell::new(DefaultEditor::new()?),
            context,
            history_file,
        })
    }
}

impl Input for ShellInput {
    fn get_command(&self) -> Result<Option<(Command, String)>, InputError> {
        let prompt = match &self.context.read().unwrap().get_prompt() {
            crate::app::context::Prompt::Ready => format!("SQL{}", INDICATOR),
            crate::app::context::Prompt::Connected(c) => {
                format!("{c}{}", INDICATOR).green().to_string()
            }
        };
        let input = self.rl.borrow_mut().readline(&prompt)?;
        let command = match parse_command(&input) {
            Ok(command) => Some((command, input)),
            Err(ParseError::Empty) => None,
            Err(err) => return Err(err.into()),
        };

        Ok(command)
    }

    fn line(&self, prompt: &str) -> Result<String, InputError> {
        let mut rl = self.rl2.borrow_mut();
        let input = rl.readline(prompt);
        input.map_err(InputError::from)
    }
}

impl Drop for ShellInput {
    fn drop(&mut self) {
        let _ = self.rl.borrow_mut().append_history(&self.history_file);
    }
}
