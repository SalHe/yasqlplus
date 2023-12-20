use std::{cell::RefCell, io::Write, process::Stdio, rc::Rc};

use colored::Colorize;
use helper::YspHelper;
use rustyline::{
    error::ReadlineError, history::FileHistory, Cmd, CompletionType, Config, EditMode, Editor,
    EventHandler, KeyEvent,
};
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

use crate::command::{self, parse_command, Command, InternalCommand, ParseError};

use self::{states::States, table::ColumnWrapper};

mod completer;
mod helper;
mod highlight;
mod states;
mod table;
mod validate;

const HISTORY_FILE: &str = "yasqlplus-history.txt";

pub struct App {
    connection: Rc<RefCell<Option<Connection>>>,
    prompt_conn: String,
    rl: Editor<YspHelper, FileHistory>,
    states: States,
    exit: bool,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        let config = Config::builder()
            .history_ignore_space(true)
            .completion_type(CompletionType::Circular)
            .edit_mode(EditMode::Vi)
            .auto_add_history(true)
            .history_ignore_dups(true)?
            .build();
        let mut rl = Editor::with_config(config)?;
        let connection = Rc::new(RefCell::new(None));
        rl.set_helper(Some(YspHelper::new(connection.clone())));
        rl.bind_sequence(KeyEvent::alt('s'), EventHandler::Simple(Cmd::Newline));
        let _ = rl.load_history(HISTORY_FILE);
        Ok(App {
            rl,
            connection,
            states: States::default(),
            prompt_conn: Default::default(),
            exit: false,
        })
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        while !self.exit {
            self.step(None)?;
        }
        Ok(())
    }

    pub fn step(&mut self, command: Option<Command>) -> anyhow::Result<()> {
        match command {
            Some(command) => self.states.command = Some(command),
            None => {
                if let Err(err) = self.get_command() {
                    match err.downcast::<ReadlineError>() {
                        Ok(rl_err) => match rl_err {
                            ReadlineError::Eof => {
                                // EOF/Ctrl+D ==> exit
                                self.exit = true;
                                return Ok(());
                            }
                            ReadlineError::Interrupted => return Ok(()), // Ctrl + C ==> next command
                            _ => return Err(rl_err.into()),
                        },
                        Err(err) => {
                            println!("Error command: {err}");
                            return Ok(());
                        }
                    };
                }
            }
        }

        if self.states.command.is_none() {
            return Ok(());
        }

        let command = self.states.command.as_ref().unwrap();
        match command {
            Command::Internal(InternalCommand::Connect(command::Connection {
                host,
                port,
                username,
                password,
            })) => {
                match self.connect(host.clone(), *port, username.clone(), password.clone()) {
                    Ok(_) => println!("Connected!"),
                    Err(err) => println!("Failed to connect: \n{err}"),
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

        match self.connection.borrow().as_ref() {
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

    fn get_prompt(&self) -> String {
        match self.connection.borrow().as_ref() {
            Some(_) => self.prompt_conn.clone().green().to_string(),
            None => "SQL > ".to_owned(),
        }
    }

    fn connect(
        &mut self,
        host: Option<String>,
        port: Option<u16>,
        username: Option<String>,
        password: Option<String>,
    ) -> anyhow::Result<()> {
        let host = match host {
            Some(v) => v,
            None => "127.0.0.1".to_owned(),
        };
        let port = port.unwrap_or(1688);
        let username = match username {
            Some(v) => v.clone(),
            None => self.normal_input("Username: ")?,
        };
        let password = match password {
            Some(v) => v.clone(),
            None => self.normal_input("Password: ")?,
        };
        match Connection::connect(&host, port, &username, &password) {
            Ok(conn) => {
                self.prompt_conn = format!("{username}@{host}:{port} > ");
                self.connection.replace(Some(conn))
            }
            Err(err) => {
                self.connection.replace(None);
                return Err(err.into());
            }
        };
        Ok(())
    }

    fn normal_input(&mut self, prompt: &str) -> Result<String, ReadlineError> {
        self.rl.helper_mut().unwrap().disable_validation();
        let input = self.rl.readline(prompt);
        self.rl.helper_mut().unwrap().enable_validation();
        input
    }

    fn get_command(&mut self) -> anyhow::Result<()> {
        let input = self.rl.readline(&self.get_prompt())?;
        self.states.command = match parse_command(&input) {
            Ok(command) => Some(command),
            Err(ParseError::Empty) => None,
            Err(err) => return Err(err.into()),
        };

        Ok(())
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

impl Drop for App {
    fn drop(&mut self) {
        let _ = self.rl.append_history(HISTORY_FILE);
    }
}
