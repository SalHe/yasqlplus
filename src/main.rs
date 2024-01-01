use std::{rc::Rc, sync::RwLock};

use app::{context::Context, input::ShellInput, AppError};
use command::{parse_connection_string, Command, Connection, InternalCommand};

mod app;
mod command;

fn main() -> Result<(), AppError> {
    let ctx = Rc::new(RwLock::new(Context::default()));
    let input = Box::new(ShellInput::new(ctx.clone())?);
    let output = Box::new(std::io::stdout());
    // let input = Box::new(BufReaderInput::new(BufReader::new(File::open("./input")?)));
    // let output = Box::new(std::fs::File::create("./sqlout")?);

    let mut app = app::App::new(input, output, ctx)?;
    let conn = std::env::args().nth(1).unwrap_or_default();
    if let Ok(connection) = parse_connection_string(&conn) {
        let Connection {
            host,
            port,
            username,
            password,
        } = &connection;
        if host.is_some() || port.is_some() || username.is_some() || password.is_some() {
            let _ = app.step(Some((
                Command::Internal(InternalCommand::Connect(connection)),
                conn,
            )));
        }
    }

    app.run()
}
