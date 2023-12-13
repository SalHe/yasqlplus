use app::{parse_connection_string, Command};

mod app;

fn main() -> anyhow::Result<()> {
    let mut app = app::App::new()?;
    if let Ok(command) = parse_connection_string(&std::env::args().nth(1).unwrap_or_default()) {
        if let Command::Connection {
            host,
            port,
            username,
            password,
        } = &command
        {
            if host.is_some() || port.is_some() || username.is_some() || password.is_some() {
                let _ = app.step(Some(command));
            }
        }
    }

    app.run()
}
