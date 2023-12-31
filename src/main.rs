use std::{rc::Rc, sync::RwLock};

use app::{context::Context, input::ShellInput, AppError};
use command::{parse_connection_string, Command, Connection, InternalCommand};

mod app;
mod command;

fn main() -> Result<(), AppError> {
    let ctx = Rc::new(RwLock::new(Context::default()));
    let input = ShellInput::new(ctx.clone())?;

    let mut app = app::App::new(Box::new(input), ctx)?;
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
