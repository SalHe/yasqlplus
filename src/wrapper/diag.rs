use std::{ffi::CStr, fmt::Display, ptr::null_mut};

use crate::native::{yacGetDiagRec, EnYacResult_YAC_ERROR, YacTextPos};

#[derive(Debug)]
pub struct DiagInfo {
    pub message: String,
    pub sql_state: String,
    pub code: i32,
    pub pos: (i32, i32),
    pub sql: Option<String>,
}

impl Display for DiagInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.pos {
            (0, 0) => write!(f, "{:?}", self),
            (line, column) => match &self.sql {
                Some(sql) => {
                    let mut lines = sql.lines().collect::<Vec<_>>();
                    let indent = " ".repeat((column - 1) as _);
                    let message = &self.message;
                    let indicator = format!("{indent}^ {message}");
                    lines.insert(line as _, &indicator);
                    write!(f, "{}", lines.join("\n"))
                }
                None => write!(f, "{:?}", self),
            },
        }
    }
}

pub fn get_error(sql: Option<String>) -> Option<DiagInfo> {
    const BUFFER_LEN: usize = 4096;
    let message = [0u8; BUFFER_LEN];
    let sql_state = [0u8; BUFFER_LEN];
    let err_code = 0i32;
    let pos = YacTextPos {
        line: -1,
        column: -1,
    };
    if EnYacResult_YAC_ERROR
        == unsafe {
            yacGetDiagRec(
                &err_code as *const _ as *mut _,
                &message as *const _ as *mut _,
                BUFFER_LEN as _,
                null_mut(),
                &sql_state as *const _ as *mut _,
                BUFFER_LEN as _,
                &pos as *const _ as *mut _,
            )
        }
    {
        None
    } else {
        let message = CStr::from_bytes_until_nul(&message[..])
            .ok()?
            .to_str()
            .ok()?
            .to_string();
        let lines = message.lines().collect::<Vec<_>>();
        let (message, pos) = if matches!(lines.first(), Some(&"PL/SQL compiling errors:")) {
            let (pos, message) = lines.get(1).unwrap().split_once(' ').unwrap();
            let pos = pos
                .trim_start_matches('[')
                .trim_end_matches(']')
                .split_once(':')
                .unwrap();
            let pos = (pos.0.parse::<i32>().unwrap(), pos.1.parse::<i32>().unwrap());
            (message.to_string(), pos)
        } else {
            (message, (pos.line, pos.column))
        };

        let sql_state = CStr::from_bytes_until_nul(&sql_state[..])
            .ok()?
            .to_str()
            .ok()?
            .to_string();
        Some(DiagInfo {
            message,
            sql_state,
            code: err_code,
            pos,
            sql,
        })
    }
}
