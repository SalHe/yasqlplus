use command::{parse_connection_string, Command, Connection, InternalCommand};

mod app;
mod command;

fn main() -> anyhow::Result<()> {
    let mut app = app::App::new()?;
    if let Ok(connection) = parse_connection_string(&std::env::args().nth(1).unwrap_or_default()) {
        let Connection {
            host,
            port,
            username,
            password,
        } = &connection;
        if host.is_some() || port.is_some() || username.is_some() || password.is_some() {
            let _ = app.step(Some(Command::Internal(InternalCommand::Connect(
                connection,
            ))));
        }
    }

    app.run()
}
