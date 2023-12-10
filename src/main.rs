use inquire::{
    set_global_render_config,
    ui::{Color, RenderConfig, Styled},
    CustomType, Password, Text,
};
use tabled::{settings::Style, Table};
use yasqlplus::wrapper::{get_connect_info, Connection, Error, Executed};

fn create_connection() -> Result<Connection, Error> {
    let conf = get_connect_info();

    let host = conf.host.unwrap_or(
        Text::new("YashanDB host")
            .with_default("127.0.0.1")
            .prompt()
            .unwrap(),
    );
    let port = conf.port.unwrap_or(
        CustomType::<u16>::new("YashanDB port")
            .with_default(1688)
            .prompt()
            .unwrap(),
    );
    let username = conf.username.unwrap_or(
        Text::new("YashanDB username")
            .with_default("sys")
            .prompt()
            .unwrap(),
    );
    let password = conf.password.unwrap_or(
        Password::new("YashanDB password")
            .without_confirmation()
            .with_display_mode(inquire::PasswordDisplayMode::Masked)
            .prompt()
            .unwrap(),
    );

    Connection::connect(&host, port, &username, &password)
}

fn get_sql() -> anyhow::Result<(String, bool)> {
    let sql = Text::new("").prompt()?;
    if sql.to_lowercase().starts_with("desc ") {
        let table_or_view = sql.split_once(' ').unwrap().1;
        Ok((format!("select * from {table_or_view} where 1 = 2"), true))
    } else {
        Ok((sql, false))
    }
}

fn app() -> anyhow::Result<()> {
    let mut connection = create_connection()?;
    let mut error_occured = false;
    loop {
        {
            // update prompt color
            let prompt = "SQL >";
            let mut config = RenderConfig::default().with_prompt_prefix(
                Styled::new(prompt).with_fg(if error_occured {
                    Color::LightRed
                } else {
                    Color::LightGreen
                }),
            );
            config.answered_prompt_prefix = Styled::new(prompt);
            config.canceled_prompt_indicator = Styled::new(prompt).with_fg(Color::Grey);
            set_global_render_config(config);
            error_occured = false;
        }

        let (sql, desc) = get_sql()?;
        if sql.is_empty() {
            continue;
        }

        if sql.to_lowercase() == "conn" {
            set_global_render_config(RenderConfig::default());
            connection = create_connection()?;
            continue;
        }

        let statment = match connection.create_statment() {
            Ok(stmt) => stmt,
            Err(err) => {
                println!("{err}");
                error_occured = true;
                continue;
            }
        };
        let result = match statment.execute_sql(&sql) {
            Ok(executed) => executed,
            Err(err) => {
                println!("{err}");
                error_occured = true;
                continue;
            }
        };
        let resolved = match result.resolve() {
            Ok(resolved) => resolved,
            Err(err) => {
                println!("{err}");
                error_occured = true;
                continue;
            }
        };

        match resolved {
            Executed::DQL(result) => {
                let columns = result.iter_columns().collect::<Vec<_>>();
                let (mut table, rows) = if desc {
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
    }
}

fn main() -> anyhow::Result<()> {
    app()
}
