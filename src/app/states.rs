#[derive(Default)]
pub struct States {
    pub command: Option<Command>,
}

#[allow(clippy::upper_case_acronyms)]
pub enum Command {
    SQL(String),
    Shell(String),
    Describe(String),
    Connection {
        host: Option<String>,
        port: Option<u16>,
        username: Option<String>,
        password: Option<String>,
    },
}
