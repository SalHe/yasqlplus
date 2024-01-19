use yasqlplus_client::wrapper::Connection;

use crate::command::Command;

#[derive(Default)]
pub struct Context {
    connection: Option<ConnectionWrapper>,
    prompt_conn: Prompt,
    last_command: Option<Command>,
    need_echo: bool,
    less_enabled: bool,
}

pub struct ConnectionWrapper(pub Connection);

unsafe impl Sync for ConnectionWrapper {}
unsafe impl Send for ConnectionWrapper {}

#[derive(Debug, Clone)]
pub enum Prompt {
    Ready,
    Connected(String),
}

impl Default for Prompt {
    fn default() -> Self {
        Self::Ready
    }
}

impl ToString for Prompt {
    fn to_string(&self) -> String {
        match self {
            Prompt::Ready => "SQL",
            Prompt::Connected(c) => c,
        }
        .to_string()
    }
}

impl Context {
    pub fn get_prompt(&self) -> Prompt {
        self.prompt_conn.clone()
    }

    pub fn set_prompt(&mut self, prompt: Prompt) {
        self.prompt_conn = prompt;
    }

    pub fn get_connection(&self) -> Option<&Connection> {
        self.connection.as_ref().map(|x| &x.0)
    }

    pub fn set_connection(&mut self, conn: Option<Connection>) {
        self.connection = conn.map(ConnectionWrapper);
    }

    pub fn get_command(&self) -> &Option<Command> {
        &self.last_command
    }

    pub fn set_command(&mut self, command: Option<Command>) {
        self.last_command = command;
    }

    pub fn need_echo(&self) -> bool {
        self.need_echo
    }

    pub fn set_need_echo(&mut self, need_echo: bool) {
        self.need_echo = need_echo;
    }

    pub fn less_enabled(&self) -> bool {
        self.less_enabled
    }

    pub fn set_less_enabled(&mut self, less_enabled: bool) {
        self.less_enabled = less_enabled;
    }
}
