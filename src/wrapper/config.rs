use std::env;

pub struct ConnectInfo {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

pub fn get_connect_info() -> anyhow::Result<ConnectInfo> {
    let host = env::var("YASHANDB_HOST")?;
    let port: u16 = env::var("YASHANDB_PORT")?.parse().unwrap();
    let username = env::var("YASHANDB_USERNAME")?;
    let password = env::var("YASHANDB_PASSWORD")?;
    Ok(ConnectInfo {
        host,
        port,
        username,
        password,
    })
}
