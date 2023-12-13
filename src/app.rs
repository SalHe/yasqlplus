use std::{cmp::max, io::Write, process::Stdio};

use helper::YspHelper;
use rustyline::{
    error::ReadlineError, history::FileHistory, Cmd, CompletionType, Config, EditMode, Editor,
    EventHandler, KeyEvent,
};
use tabled::{settings::Style, Table};
use terminal_size::{terminal_size, Height, Width};
use yasqlplus::wrapper::{Connection, Executed, LazyExecuted};

use self::states::States;

mod conn_str;
mod helper;
mod highlight;
mod states;
mod validate;

pub use conn_str::parse_connection_string;
pub use states::Command;

const HISTORY_FILE: &str = "yasqlplus-history.txt";

pub struct App {
    connection: Option<Connection>,
    rl: Editor<YspHelper, FileHistory>,
    states: States,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        let config = Config::builder()
            .history_ignore_space(true)
            .completion_type(CompletionType::List)
            .edit_mode(EditMode::Vi)
            .build();
        let mut rl = Editor::with_config(config)?;
        rl.set_helper(Some(YspHelper::new()));
        rl.bind_sequence(KeyEvent::alt('s'), EventHandler::Simple(Cmd::Newline));
        let _ = rl.load_history(HISTORY_FILE);
        Ok(App {
            rl,
            connection: None,
            states: States::default(),
        })
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        loop {
            self.step(None)?;
        }
    }

    pub fn step(&mut self, command: Option<Command>) -> anyhow::Result<()> {
        match command {
            Some(command) => self.states.command = Some(command),
            None => {
                if let Err(err) = self.get_command() {
                    match err.downcast::<ReadlineError>() {
                        Ok(rl_err) => return Err(rl_err.into()),
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
            Command::Connection {
                host,
                port,
                username,
                password,
            } => {
                match self.connect(
                    host.clone(),
                    port.clone(),
                    username.clone(),
                    password.clone(),
                ) {
                    Ok(_) => println!("Connected!"),
                    Err(err) => println!("Failed to connect: {err}"),
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

        match &self.connection {
            Some(connection) => {
                let result = self.execute_command(&connection, command);
                match result {
                    Ok(result) => match self.show_result(result, command) {
                        Ok(_) => {}
                        Err(err) => println!("{err}"),
                    },
                    Err(err) => println!("{err}"),
                }
            }
            None => {
                println!("Not connected!");
            }
        }

        Ok(())
    }

    fn show_result(&self, result: LazyExecuted, command: &Command) -> anyhow::Result<()> {
        let resolved = result.resolve()?;
        match resolved {
            Executed::DQL(result) => {
                let columns = result.iter_columns().collect::<Vec<_>>();
                let (mut table, rows) = if matches!(command, Command::Describe(_)) {
                    (Table::new(columns), None)
                } else {
                    let mut builder = tabled::builder::Builder::default();
                    let rows = result.rows().collect::<Vec<_>>();
                    rows.iter().for_each(|row| {
                        builder.push_record(row.iter().map(|x| format!("{x}")));
                    });
                    builder.insert_record(0, columns.iter().map(|x| x.name.clone()));
                    (builder.build(), Some(rows.len()))
                };

                if matches!(rows, None) || matches!(rows, Some(row) if row > 0) {
                    let table = table.with(Style::rounded());
                    self.show_long_if_necessary(&table.to_string());
                    println!("{table}");
                }

                if let Some(rows) = rows {
                    println!(
                        "{rows} row{suffix} fetched",
                        suffix = if rows > 0 { "s" } else { "" }
                    );
                }
            }
            Executed::DML(affection) => {
                println!("{} row(s) affected", affection.affected())
            }
            Executed::DCL(_instrction) => println!("DCL exectued"),
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
            if console::measure_text_width(content) >= w as _ {
                match std::process::Command::new("less")
                    .arg("-S")
                    .stdin(Stdio::piped())
                    .spawn()
                {
                    Ok(mut command) => {
                        if let Some(mut stdin) = command.stdin.take() {
                            let _ = stdin.write_all(content.as_bytes());
                        }
                        let _ = command.wait();
                    }
                    Err(_) => {}
                }
            }
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
        Ok(
            match Connection::connect(&host, port, &username, &password) {
                Ok(conn) => self.connection = Some(conn),
                Err(err) => println!("Failed to connect: {}", err),
            },
        )
    }

    fn normal_input(&mut self, prompt: &str) -> Result<String, ReadlineError> {
        self.rl.helper_mut().unwrap().disable_validation();
        let input = self.rl.readline(prompt);
        self.rl.helper_mut().unwrap().enable_validation();
        input
    }

    fn parse_command(input: &str) -> anyhow::Result<Option<Command>> {
        let command = if input.is_empty() {
            None
        } else if input.starts_with('!') {
            Some(Command::Shell(input[1..].to_owned()))
        } else if input.to_lowercase().starts_with("desc ") {
            let table_or_view = input.split_once(' ').unwrap().1;
            Some(Command::Describe(table_or_view.to_owned()))
        } else if input.to_lowercase().starts_with("conn ") {
            Some(parse_connection_string(input.split_once(' ').unwrap().1)?)
        } else {
            Some(Command::SQL(input.to_owned()))
        };
        Ok(command)
    }

    fn get_command(&mut self) -> anyhow::Result<()> {
        let input = self.rl.readline("SQL > ")?;
        let _ = self.rl.add_history_entry(&input);
        self.states.command = App::parse_command(&input)?;

        Ok(())
    }

    fn execute_command(
        &self,
        connection: &Connection,
        command: &Command,
    ) -> anyhow::Result<LazyExecuted> {
        let statment = connection.create_statment()?;

        let sql = match command {
            Command::SQL(sql) => {
                if sql.is_empty() || !sql.ends_with([';', '/']) {
                    sql.to_owned()
                } else {
                    // trim trailing ';' for statement
                    //               '/' for block
                    sql[..sql.len() - 1].to_owned()
                }
            }
            Command::Describe(table_or_view) => format!(
                "select * from {table_or_view} where 1=2",
                table_or_view = &table_or_view[..(max(table_or_view.len() - 1, 0))]
            ),
            Command::Connection { .. } => unreachable!("Connecting should be processed before."),
            Command::Shell(_) => unreachable!("Shell command should be processed before."),
        };
        let result = statment.execute_sql(&sql)?;
        Ok(result)
    }
}

impl Drop for App {
    fn drop(&mut self) {
        let _ = self.rl.append_history(HISTORY_FILE);
    }
}
