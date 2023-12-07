use crate::native::{yacDirectExecute, yacPrepare, EnYacResult_YAC_ERROR};

use super::{Error, PreparedStatement, ResultSet, StatementHandle};

pub struct Statement(StatementHandle);

impl Statement {
    pub fn execute_sql(self, sql: &str) -> Result<ResultSet, Error> {
        // let sql = CString::new(sql).map_err(|err| Error::Other(err.into()))?;
        if unsafe { yacDirectExecute(self.0 .0, sql.as_ptr() as _, sql.len() as _) }
            == EnYacResult_YAC_ERROR
        {
            return Err(Error::get_yas_diag(Some(sql.to_string())).unwrap());
        }

        Ok(ResultSet::new(self.0))
    }

    pub fn prepare(self, sql: &str) -> Result<PreparedStatement, Error> {
        if EnYacResult_YAC_ERROR
            == unsafe { yacPrepare(self.0 .0, sql.as_ptr() as _, sql.len() as _) }
        {
            return Err(Error::get_yas_diag(Some(sql.to_string())).unwrap());
        }
        Ok(PreparedStatement(self.0))
    }
}

impl From<StatementHandle> for Statement {
    fn from(value: StatementHandle) -> Self {
        Self(value)
    }
}
