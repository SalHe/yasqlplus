use anyhow::anyhow;

use super::Command;

pub fn parse_connection_string(conn: &str) -> anyhow::Result<Command> {
    enum State {
        Username,
        Password,
        Host,
        Port,
    }

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
                    return Err(anyhow!("Unsupported connection string format"));
                }
            }
            last_index = id + 1;
        }
    }

    if conn.len() + 1 != last_index {
        // SEP_END
        Err(anyhow!("Unsupported connection string format"))
    } else {
        let process = |x: Option<String>| x.and_then(|x| if x.is_empty() { None } else { Some(x) });
        Ok(Command::Connection {
            host: process(host),
            port: match process(port) {
                Some(s) => match s.parse() {
                    Ok(port) => Some(port),
                    Err(err) => return Err(err.into()),
                },
                None => None,
            },
            username: process(username),
            password: process(password),
        })
    }
}
