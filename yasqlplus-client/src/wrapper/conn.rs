use crate::native::{yacConnect, yacDisconnect, EnYacResult_YAC_ERROR};

use super::{DbcHandle, EnvHandle, Error, Statement, StatementHandle};

pub struct Connection {
    conn_handle: DbcHandle,
    _env_handle: EnvHandle, // must be dropped after `conn_handle`
}

impl Connection {
    pub fn connect(host: &str, port: u16, username: &str, password: &str) -> Result<Self, Error> {
        let env_handle = EnvHandle::new()?.with_utf8();
        let conn_handle = DbcHandle::new(&env_handle)?;

        let url = format!("{host}:{port}");
        let result = unsafe {
            yacConnect(
                conn_handle.0,
                url.as_ptr() as _,
                url.len() as _,
                username.as_ptr() as _,
                username.len() as _,
                password.as_ptr() as _,
                password.len() as _,
            )
        };

        if result == EnYacResult_YAC_ERROR {
            return Err(Error::get_yas_diag(None).unwrap());
        }

        Ok(Self {
            _env_handle: env_handle,
            conn_handle,
        })
    }

    pub fn create_statement(&self) -> Result<Statement, Error> {
        let stmt = StatementHandle::new(&self.conn_handle)?;
        Ok(stmt.into())
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        unsafe { yacDisconnect(self.conn_handle.0) }
    }
}

#[cfg(test)]
mod test {
    use std::env;

    use crate::wrapper::{config::get_connect_info, Connection, Error};

    #[test]
    fn connect_properly() {
        let host = env::var("YASHANDB_HOST")
            .expect("please specify yashandb host via environment variable `YASHANDB_HOST`");
        let port: u16 = env::var("YASHANDB_PORT")
            .expect("please specify yashandb port via environment variable `YASHANDB_PORT`")
            .parse()
            .unwrap();
        let username = env::var("YASHANDB_USERNAME").expect(
            "please specify yashandb username via environment variable `YASHANDB_USERNAME`",
        );
        let password = env::var("YASHANDB_PASSWORD").expect(
            "please specify yashandb password via environment variable `YASHANDB_PASSWORD`",
        );
        assert!(Connection::connect(&host, port, &username, &password).is_ok());
    }

    macro_rules! fail_test {
        ($test:ident: $host:expr, $port:expr, $username:expr, $password:expr => $error:expr) => {
            #[test]
            fn $test() {
                assert!(
                    Connection::connect($host, $port, $username, $password).is_err_and(|e| {
                        if let Error::YasClient(diag) = e {
                            diag.message.contains($error)
                        } else {
                            false
                        }
                    })
                );
            }
        };
    }

    fail_test!(fail_to_connect_socket: "127.0.0.1", 9999, "hello", "world" => "failed to connect socket");
    fail_test!(wrong_authentication:
        &get_connect_info().host.unwrap(),
        get_connect_info().port.unwrap(),
        "invalid username", "invalid password"
        => "invalid username/password");
}
