#[derive(Default)]
pub struct States {
    pub command: Option<Command>,
}

pub enum Command {
    SQL(String),
    Describe(String),
    Connection {
        host: String,
        port: u16,
        username: String,
        password: String,
    },
}
