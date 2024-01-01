use std::{fs::File, io::BufReader, rc::Rc, sync::RwLock};

use app::{
    context::Context,
    input::{BufReaderInput, Input, ShellInput},
    output::Output,
    AppError,
};
use clap::Parser;
use command::{parse_connection_string, Command, Connection, InternalCommand};

mod app;
mod command;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about=None)]
struct Cli {
    /// Connection string.
    ///
    ///     [<username>]/[<password>][@[<host>]:[<port>]]
    ///
    /// <host> will defaults to `127.0.0.1`.
    /// <port> will defaults to `1688`.
    ///
    /// Leave <username> or <password> empty for inputting from next line.
    /// To specify complex username/password you could leave them empty.
    /// Or override them via cli arguments like `--username "complex username"`
    ///
    /// For example:
    ///     sys/yasdb_123@127.0.0.1:1688
    ///     sys/yasdb_123@:1601
    conn: Vec<String>,

    /// Database username.
    #[arg(short, long)]
    username: Option<String>,

    /// Database password.
    #[arg(short, long)]
    password: Option<String>,

    /// Database host.
    #[arg(long)]
    host: Option<String>,

    /// Database port.
    #[arg(long)]
    port: Option<u16>,

    /// Echo commands executed.
    #[arg(short, long)]
    echo: bool,

    /// SQL scripts file.
    #[arg(short, long)]
    input: Option<String>,

    /// Result output file.
    #[arg(short, long)]
    output: Option<String>,
}

fn main() -> Result<(), AppError> {
    let args = Cli::parse();

    let ctx = Rc::new(RwLock::new(Context::default()));
    let input: Box<dyn Input> = match args.input {
        // TODO support network file(e.g. http/https)
        Some(input) => Box::new(BufReaderInput::new(
            BufReader::new(File::open(input)?),
            args.echo,
        )),
        None => Box::new(ShellInput::new(ctx.clone())?),
    };
    let output: Box<dyn Output> = match args.output {
        Some(output) => Box::new(File::create(output)?),
        None => Box::new(std::io::stdout()),
    };

    let mut app = app::App::new(input, output, ctx)?;

    // Parse connection string.
    let conn_str = args.conn.first().cloned().unwrap_or_default();
    let connection = parse_connection_string(&conn_str).unwrap_or_default();
    // Override from cli arguments.
    let connection = Connection {
        username: args.username.or(connection.username),
        password: args.password.or(connection.password),
        host: args.host.or(connection.host),
        port: args.port.or(connection.port),
    };
    if connection.any_valid() {
        let _ = app.step(Some((
            Command::Internal(InternalCommand::Connect(connection)),
            conn_str,
        )));
    }

    app.run()
}
