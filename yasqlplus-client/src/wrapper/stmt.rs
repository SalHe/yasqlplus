use crate::native::{yacDirectExecute, yacPrepare, EnYacResult_YAC_ERROR};

use super::{Error, LazyExecuted, PreparedStatement, StatementHandle};

pub struct Statement(StatementHandle);

impl Statement {
    pub fn execute_sql(self, sql: &str) -> Result<LazyExecuted, Error> {
        // let sql = CString::new(sql).map_err(|err| Error::Other(err.into()))?;
        if unsafe { yacDirectExecute(self.0 .0, sql.as_ptr() as _, sql.len() as _) }
            == EnYacResult_YAC_ERROR
        {
            Err(Error::get_yas_diag(Some(sql.to_string())).unwrap())
        } else {
            Ok(LazyExecuted(self.0))
        }
    }

    pub fn prepare(self, sql: &str) -> Result<PreparedStatement, Error> {
        if EnYacResult_YAC_ERROR
            == unsafe { yacPrepare(self.0 .0, sql.as_ptr() as _, sql.len() as _) }
        {
            Err(Error::get_yas_diag(Some(sql.to_string())).unwrap())
        } else {
            Ok(PreparedStatement(self.0))
        }
    }
}

impl From<StatementHandle> for Statement {
    fn from(value: StatementHandle) -> Self {
        Self(value)
    }
}
