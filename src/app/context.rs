use colored::Colorize;
use yasqlplus_client::wrapper::Connection;

use crate::command::Command;

#[derive(Default)]
pub struct Context {
    connection: Option<Connection>,
    prompt_conn: String,
    last_command: Option<Command>,
}

impl Context {
    pub fn get_prompt(&self) -> String {
        if self.connection.is_none() {
            "SQL > ".to_owned()
        } else {
            self.prompt_conn.green().to_string()
        }
    }

    pub fn set_prompt(&mut self, prompt: String) {
        self.prompt_conn = prompt;
    }

    pub fn get_connection(&self) -> &Option<Connection> {
        &self.connection
    }

    pub fn set_connection(&mut self, conn: Option<Connection>) {
        self.connection = conn;
    }

    pub fn get_command(&self) -> &Option<Command> {
        &self.last_command
    }

    pub fn set_command(&mut self, command: Option<Command>) {
        self.last_command = command;
    }
}
