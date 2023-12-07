use yasqlplus::wrapper::{get_connect_info, Connection};

fn main() -> anyhow::Result<()> {
    let conf = get_connect_info()?;
    let connection = Connection::connect(&conf.host, conf.port, &conf.username, &conf.password)?;
    let statment = connection.create_statment()?;

    let sql = std::env::args()
        .nth(1)
        .unwrap_or("select * from dba_tables where 1 = 2".to_string());

    let result = statment.execute_sql(&sql)?;

    for col in result.iter_columns()? {
        println!("{:?}", col);
    }
    Ok(())
}
