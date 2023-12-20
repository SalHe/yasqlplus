use std::env;

pub struct ConnectInfo {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
}

pub fn get_connect_info() -> ConnectInfo {
    let host = env::var("YASHANDB_HOST").ok();
    let port = env::var("YASHANDB_PORT").ok().and_then(|x| x.parse().ok());
    let username = env::var("YASHANDB_USERNAME").ok();
    let password = env::var("YASHANDB_PASSWORD").ok();
    ConnectInfo {
        host,
        port,
        username,
        password,
    }
}
