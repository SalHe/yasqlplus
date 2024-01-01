use std::num::ParseIntError;

use thiserror::Error;

#[derive(Debug, Default)]
pub struct Connection {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl Connection {
    pub fn any_valid(&self) -> bool {
        self.host.is_some()
            || self.port.is_some()
            || self.username.is_some()
            || self.password.is_some()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Username,
    Password,
    Host,
    Port,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConnParsingError {
    #[error("Failed to parsing port: {0}")]
    Port(ParseIntError),

    #[error("Desired: {0:?}")]
    Expected(State),

    #[error("Invalid format.")]
    Invalid,
}

pub fn parse_connection_string(conn: &str) -> Result<Connection, ConnParsingError> {
    // TODO support parse role (e.g. sys/xxx as sysdba)
    let mut last_index = 0;
    let mut state = State::Username;
    let mut username = None;
    let mut password = None;
    let mut host = None;
    let mut port = None;

    const SEP_PASSWORD: char = '/';
    const SEP_HOST: char = '@';
    const SEP_PORT: char = ':';
    const SEP_END: char = '\x01';

    for (id, ch) in conn.chars().chain([SEP_END]).enumerate() {
        if [SEP_PASSWORD, SEP_HOST, SEP_PORT, SEP_END].contains(&ch) {
            let s = &conn[last_index..id];

            match (ch, state) {
                (SEP_PASSWORD, State::Username) => {
                    username = Some(s.to_owned());
                    state = State::Password;
                }
                (SEP_HOST, State::Username) => {
                    username = Some(s.to_owned());
                    state = State::Host;
                }
                (SEP_HOST, State::Password) => {
                    password = Some(s.to_owned());
                    state = State::Host;
                }
                (SEP_PORT, _) => {
                    host = Some(s.to_owned());
                    state = State::Port;
                }
                (SEP_END, stat) => {
                    match stat {
                        State::Username => username = Some(s.to_owned()),
                        State::Password => password = Some(s.to_owned()),
                        State::Host => host = Some(s.to_owned()),
                        State::Port => port = Some(s.to_owned()),
                    }
                    last_index = id + 1;
                    break;
                }
                (_, _) => {
                    return Err(ConnParsingError::Expected(state));
                }
            }
            last_index = id + 1;
        }
    }

    if conn.len() + 1 != last_index {
        // SEP_END
        Err(ConnParsingError::Invalid)
    } else {
        let process = |x: Option<String>| x.and_then(|x| if x.is_empty() { None } else { Some(x) });
        Ok(Connection {
            host: process(host),
            port: match process(port) {
                Some(s) => match s.parse() {
                    Ok(port) => Some(port),
                    Err(err) => return Err(ConnParsingError::Port(err)),
                },
                None => None,
            },
            username: process(username),
            password: process(password),
        })
    }
}
