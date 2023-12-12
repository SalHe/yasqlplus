#[derive(Default)]
pub struct States {
    pub command: Option<Command>,
}

pub enum Command {
    SQL(String),
    Describe(String),
    Connection {
        host: Option<String>,
        port: Option<u16>,
        username: Option<String>,
        password: Option<String>,
    },
}
