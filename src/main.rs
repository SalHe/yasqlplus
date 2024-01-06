use std::{
    fs::File,
    io::BufReader,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use app::{
    context::Context,
    input::{BufReaderInput, Input, Reed, ShellInput, SingleCommand},
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
    ///     [<username>][/[<password>][@[<host>][:[<port>]]]]
    ///
    /// <host> will default to `127.0.0.1`.
    /// <port> will default to `1688`.
    ///
    /// Leave <username> or <password> empty for inputting from next line.
    /// To specify complex username/password you could leave them empty.
    /// Or override them via cli arguments like `--username "complex username"`
    ///
    /// For example:
    ///     sys/yasdb_123@127.0.0.1:1688
    ///     sys/yasdb_123@:1601
    #[arg(verbatim_doc_comment)]
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
    file: Option<String>,

    /// Single command.
    #[arg(short, long)]
    command: Option<String>,

    /// Disable show large table in less.
    #[arg(long)]
    no_less: bool,

    /// History file **directory**. Default to user home.
    /// The history file name is specified by `--history-file`.
    #[arg(short = 'H', long)]
    history_path: Option<String>,

    /// History file name.
    #[arg(short = 'F', long, default_value = "yasqlplus-history.txt")]
    history_file: String,

    /// Use reedline as line editor.
    /// EXPERIMENTAL FEATURE.
    #[arg(long, verbatim_doc_comment)]
    reedline: bool,
}

fn main() -> Result<(), AppError> {
    let args = Cli::parse();

    let mut ctx = Context::default();
    ctx.set_need_echo(args.echo);
    ctx.set_less_enabled(!args.no_less);

    let history_file = args
        .history_path
        .map(PathBuf::from)
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_default())
        .join(args.history_file)
        .to_str()
        .unwrap()
        .to_owned();

    let ctx = Arc::new(RwLock::new(ctx));
    let input: Box<dyn Input> = match args.command {
        Some(command) => Box::new(SingleCommand::new(command)),
        None => match args.file {
            // TODO support network file(e.g. http/https)
            Some(input) => Box::new(BufReaderInput::new(BufReader::new(File::open(input)?))),
            None if args.reedline => Box::new(Reed::new(ctx.clone(), history_file)?),
            None => Box::new(ShellInput::new(ctx.clone(), history_file)?),
        },
    };
    let mut app = app::App::new(input, ctx)?;

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
