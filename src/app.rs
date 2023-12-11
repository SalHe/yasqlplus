use rustyline::{
    history::FileHistory, Cmd, CompletionType, Config, EditMode, Editor, EventHandler, KeyEvent,
};
use tabled::{settings::Style, Table};
use yasqlplus::wrapper::{Connection, Executed};

use helper::YspHelper;

use self::states::{Command, States};

mod helper;
mod states;
mod validate;

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
            self.get_command()?;
            if self.states.command.is_none() {
                continue;
            }

            let command = self.states.command.as_ref().unwrap();
            match command {
                Command::Connection {
                    host,
                    port,
                    username,
                    password,
                } => {
                    match Connection::connect(host, *port, username, password) {
                        Ok(conn) => self.connection = Some(conn),
                        Err(err) => println!("Failed to connect: {}", err),
                    }
                    continue;
                }
                _ => {}
            }

            if self.connection.is_none() {
                println!("Not connected!");
                continue;
            }

            match self.execute_command(self.connection.as_ref().unwrap(), command) {
                Ok(_) => {}
                Err(err) => println!("{err}"),
            }
        }
    }

    fn get_command(&mut self) -> anyhow::Result<()> {
        let input = self.rl.readline("SQL > ")?;
        let _ = self.rl.add_history_entry(&input);
        if input.is_empty() {
            self.states.command = None;
        } else if input.to_lowercase().starts_with("desc ") {
            let table_or_view = input.split_once(' ').unwrap().1;
            self.states.command = Some(Command::Describe(table_or_view.to_owned()));
        } else if input.to_lowercase().starts_with("conn ") {
            self.states.command = Some(parse_connection_string(input.split_once(' ').unwrap().1)?);
        } else {
            self.states.command = Some(Command::SQL(input));
        }
        Ok(())
    }

    fn execute_command(&self, connection: &Connection, command: &Command) -> anyhow::Result<()> {
        let statment = connection.create_statment()?;

        let sql = match command {
            Command::SQL(sql) => sql.clone(),
            Command::Describe(table_or_view) => format!("select * from {table_or_view} where 1=2"),
            Command::Connection { .. } => {
                unreachable!("Connecting should be processed before.")
            }
        };
        let result = statment.execute_sql(&sql)?;
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
                let table = table.with(Style::rounded());
                println!("{table}");

                if let Some(rows) = rows {
                    println!("{} row(s) fetched", rows);
                }
            }
            Executed::DML(affection) => {
                println!("{} row(s) affected", affection.affected())
            }
            Executed::DCL(_instrction) => println!("DCL exectued"),
            Executed::Unknown(_) => println!("Succeed"),
        }

        Ok(())
    }
}

impl Drop for App {
    fn drop(&mut self) {
        let _ = self.rl.append_history(HISTORY_FILE);
    }
}

fn parse_connection_string(conn: &str) -> anyhow::Result<Command> {
    let mut parts = conn.split(['/', '@', ':']);
    let username = parts.next().unwrap().to_owned();
    let password = parts.next().unwrap().to_owned();
    let host = parts.next().unwrap().to_owned();
    let port = parts
        .next()
        .unwrap()
        .parse()
        .map_err(Into::<anyhow::Error>::into)?;
    Ok(Command::Connection {
        host,
        port,
        username,
        password,
    })
}
