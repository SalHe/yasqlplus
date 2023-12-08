use yasqlplus::wrapper::{get_connect_info, Connection, Executed};

fn main() -> anyhow::Result<()> {
    let conf = get_connect_info()?;
    let connection = Connection::connect(&conf.host, conf.port, &conf.username, &conf.password)?;
    let statment = connection.create_statment()?;

    let sql = std::env::args()
        .nth(1)
        .unwrap_or("select * from t1".to_string());

    let result = statment.execute_sql(&sql)?;

    match result.resolve()? {
        Executed::DQL(result) => {
            println!("Columns {}: ", result.columns());
            for col in result.iter_columns() {
                println!("    {:?}", col);
            }

            let rows = result.rows().collect::<Vec<_>>();
            println!("{rows:?}");
            println!("{} row(s) fetched", rows.len());
        }
        Executed::DML(affection) => {
            println!("Affected: {}", affection.affected())
        }
        Executed::DCL(_instrction) => println!("DCL exectued"),
        Executed::Unknown(_) => unreachable!(),
    }

    Ok(())
}
