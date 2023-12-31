use std::{io::Write, process::Stdio, rc::Rc, sync::RwLock};

use colored::Colorize;

use syntect::{
    easy::HighlightLines, highlighting::ThemeSet, parsing::SyntaxSet,
    util::as_24_bit_terminal_escaped,
};
use tabled::{
    settings::{object::Cell as TableCell, Format, Modify, Style},
    Table,
};
use terminal_size::{terminal_size, Height, Width};
use yasqlplus_client::wrapper::{Connection, DiagInfo, Error, Executed, LazyExecuted};

use crate::command::{self, Command, InternalCommand};

use self::{
    context::Context,
    input::{InputError, InputSource},
    table::ColumnWrapper,
};

mod completer;
mod helper;
mod highlight;
mod table;
mod validate;

pub mod context;
pub mod input;

pub struct App {
    context: Rc<RwLock<Context>>,
    input: Box<dyn InputSource>,
    exit: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Input error: {0}")]
    Input(#[from] InputError),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl App {
    pub fn new(
        input: Box<dyn InputSource>,
        context: Rc<RwLock<Context>>,
    ) -> Result<Self, AppError> {
        Ok(App {
            input,
            context,
            exit: false,
        })
    }

    pub fn run(&mut self) -> Result<(), AppError> {
        while !self.exit {
            self.step(None)?;
        }
        Ok(())
    }

    pub fn step(&mut self, command: Option<Command>) -> Result<(), AppError> {
        let command = match command {
            Some(command) => Some(command),
            None => {
                match self.input.get_command() {
                    Ok(command) => command,
                    Err(InputError::Eof) => {
                        // EOF/Ctrl+D ==> exit
                        self.exit = true;
                        return Ok(());
                    }
                    Err(InputError::Cancelled) => {
                        // EOF/Ctrl+D ==> exit
                        self.exit = true;
                        return Ok(());
                    }
                    Err(err) => return Err(err.into()),
                }
            }
        };
        let mut ctx = self.context.write().unwrap();
        ctx.set_command(command);

        let command = ctx.get_command();
        if command.is_none() {
            return Ok(());
        }

        let command = command.as_ref().unwrap();
        match command {
            Command::Internal(InternalCommand::Connect(command::Connection {
                host,
                port,
                username,
                password,
            })) => {
                match self.connect(host.clone(), *port, username.clone(), password.clone()) {
                    Ok((conn, prompt)) => {
                        ctx.set_connection(Some(conn));
                        ctx.set_prompt(prompt);
                        println!("Connected!");
                    }
                    Err(err) => {
                        ctx.set_connection(None);
                        println!("Failed to connect: \n{err}");
                    }
                }
                return Ok(());
            }
            Command::Shell(shell) => {
                std::process::Command::new("sh")
                    .arg("-c")
                    .arg(shell)
                    .stdin(Stdio::inherit())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .spawn()?
                    .wait()?;
                return Ok(());
            }
            _ => {}
        }

        match ctx.get_connection().as_ref() {
            Some(connection) => {
                let result = self.execute_command(connection, command);
                match result {
                    Ok(Some(result)) => match self.show_result(result, command) {
                        Ok(_) => {}
                        Err(err) => println!("{err}"),
                    },
                    Ok(None) => {} // comment
                    Err(err) => self.print_execute_sql_error(err),
                }
            }
            None => {
                println!("Not connected!");
            }
        }

        Ok(())
    }

    fn print_execute_sql_error(&self, err: Error) {
        match err {
            Error::YasClient(err) => match err.pos {
                (0, 0) => {
                    let DiagInfo {
                        message,
                        sql_state,
                        code,
                        ..
                    } = err;
                    println!(
                        "{}",
                        format!("YAS-{code:0>5}: {message} (SQL State: {sql_state})").red()
                    )
                }
                (line, column) => match &err.sql {
                    Some(sql) => {
                        if sql.is_empty() {
                            return println!("{}", err.message.red());
                        }
                        let mut lines = vec![];

                        let heading = format!("  {line} | ");
                        lines.push(format!(
                            "{heading}{code}",
                            heading = heading.blue(),
                            code = {
                                let ps = SyntaxSet::load_defaults_newlines();
                                let ts = ThemeSet::load_defaults();

                                let syntax = ps.find_syntax_by_extension("sql").unwrap();
                                let mut h =
                                    HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
                                let ranges: Vec<(syntect::highlighting::Style, &str)> = h
                                    .highlight_line(
                                        sql.lines().nth(line as usize - 1).unwrap(),
                                        &ps,
                                    )
                                    .unwrap();
                                let mut escaped = as_24_bit_terminal_escaped(&ranges[..], false);
                                escaped.push_str("\x1b[0m");
                                escaped
                            }
                        ));
                        lines.push(
                            format!(
                                "{indent}^ {message}",
                                indent = " ".repeat(heading.len() + column as usize - 1),
                                message = err.message
                            )
                            .red()
                            .to_string(),
                        );
                        println!("{}", lines.join("\n"))
                    }
                    None => println!("{:?}", err),
                },
            },
            Error::Other => todo!(),
        }
    }

    fn show_result(&self, result: LazyExecuted, command: &Command) -> anyhow::Result<()> {
        let resolved = result.resolve()?;
        match resolved {
            Executed::DQL(result) => {
                let columns = result.iter_columns().collect::<Vec<_>>();
                let (mut table, styling, rows) =
                    if matches!(command, Command::Internal(InternalCommand::Describe(_))) {
                        let styling: Box<dyn FnOnce(&mut Table)> = Box::new(|_: &mut Table| {});
                        (Table::new(columns.iter().map(ColumnWrapper)), styling, None)
                    } else {
                        let mut builder = tabled::builder::Builder::default();
                        let mut nulls = Vec::<(usize, usize)>::new();
                        let rows = result.rows().collect::<Vec<_>>();
                        rows.iter().enumerate().for_each(|(y, row)| {
                            builder.push_record(row.iter().enumerate().map(
                                |(x, value)| match value {
                                    Some(x) => format!("{x}"),
                                    None => {
                                        nulls.push((y, x));
                                        "<null>".to_owned()
                                    }
                                },
                            ));
                        });
                        builder.insert_record(0, columns.iter().map(|x| x.name.clone()));

                        let styling: Box<dyn FnOnce(&mut Table)> = Box::new(|table: &mut Table| {
                            for (row, col) in nulls {
                                let _ =
                                    &table.with(Modify::new(TableCell::new(row + 1, col)).with(
                                        Format::content(|x| x.to_owned().italic().to_string()),
                                    ));
                            }
                        });
                        (builder.build(), styling, Some(rows.len()))
                    };

                if rows.is_none() || matches!(rows, Some(row) if row > 0) {
                    let table = table.with(Style::rounded());
                    self.show_long_if_necessary(&table.to_string());
                    styling(table);
                    println!("{table}");
                }

                if let Some(rows) = rows {
                    println!("{rows} row(s) fetched");
                }
            }
            Executed::DML(affection) => {
                println!("{} row(s) affected", affection.affected())
            }
            Executed::DCL(_instruction) => println!("DCL executed"),
            Executed::Unknown(_) => println!("Succeed"),
        };
        Ok(())
    }

    fn show_long_if_necessary(&self, content: &str) {
        if !console::Term::stdout().is_term() {
            return;
        }
        let size = terminal_size();
        if let Some((Width(w), Height(_h))) = size {
            if console::measure_text_width(content.lines().nth(0).unwrap_or_default()) >= w as _ {
                if let Ok(mut command) = std::process::Command::new("less")
                    .arg("-S")
                    .stdin(Stdio::piped())
                    .spawn()
                {
                    if let Some(mut stdin) = command.stdin.take() {
                        let _ = stdin.write_all(content.as_bytes());
                    }
                    let _ = command.wait();
                }
            }
        }
    }

    fn connect(
        &self,
        host: Option<String>,
        port: Option<u16>,
        username: Option<String>,
        password: Option<String>,
    ) -> anyhow::Result<(Connection, String)> {
        let host = match host {
            Some(v) => v,
            None => "127.0.0.1".to_owned(),
        };
        let port = port.unwrap_or(1688);
        let username = match username {
            Some(v) => v.clone(),
            None => self.input.line("Username: ").unwrap_or_default(),
        };
        let password = match password {
            Some(v) => v.clone(),
            None => self.input.line("Password: ").unwrap_or_default(),
        };
        match Connection::connect(&host, port, &username, &password) {
            Ok(conn) => Ok((conn, format!("{username}@{host}:{port} > "))),
            Err(err) => Err(err.into()),
        }
    }

    fn execute_command(
        &self,
        connection: &Connection,
        command: &Command,
    ) -> Result<Option<LazyExecuted>, Error> {
        let statement = connection.create_statement()?;

        let sql = match command {
            Command::SQL(sql) => {
                if sql.lines().count() == 1 && sql.starts_with("--") {
                    return Ok(None);
                }
                if sql.is_empty() || !sql.ends_with([';', '/']) {
                    sql.to_owned()
                } else {
                    // trim trailing ';' for statement
                    //               '/' for block
                    sql[..sql.len() - 1].to_owned()
                }
            }
            Command::Internal(InternalCommand::Describe(table_or_view)) => {
                format!("select * from {table_or_view} where 1=2")
            }
            Command::Internal(InternalCommand::Connect(_)) => {
                unreachable!("Connecting should be processed before.")
            }
            Command::Shell(_) => unreachable!("Shell command should be processed before."),
        };
        let result = statement.execute_sql(&sql)?;
        Ok(Some(result))
    }
}
