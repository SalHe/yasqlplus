use std::ptr::null_mut;

use crate::native::{
    yacGetStmtAttr, yacNumResultCols, EnYacResult_YAC_ERROR, EnYacSQLType_YAC_SQLTYPE_COMMIT,
    EnYacSQLType_YAC_SQLTYPE_DELETE, EnYacSQLType_YAC_SQLTYPE_GRANT,
    EnYacSQLType_YAC_SQLTYPE_INSERT, EnYacSQLType_YAC_SQLTYPE_MERGE,
    EnYacSQLType_YAC_SQLTYPE_QUERY, EnYacSQLType_YAC_SQLTYPE_REVOKE,
    EnYacSQLType_YAC_SQLTYPE_ROLLBACK, EnYacSQLType_YAC_SQLTYPE_UPDATE,
    EnYacStmtAttr_YAC_ATTR_ROWS_AFFECTED, EnYacStmtAttr_YAC_ATTR_SQLTYPE,
};

use super::{ColumnsIterator, Error, RowsIterator, StatementHandle};

pub struct LazyExecuted(pub(crate) StatementHandle);

impl LazyExecuted {
    pub fn resolve(self) -> Result<Executed, Error> {
        let t = 0;
        if EnYacResult_YAC_ERROR
            == unsafe {
                yacGetStmtAttr(
                    self.0 .0,
                    EnYacStmtAttr_YAC_ATTR_SQLTYPE,
                    &t as *const _ as *mut _,
                    std::mem::size_of_val(&t) as _,
                    null_mut(),
                )
            }
        {
            Err(Error::get_yas_diag(None).unwrap())
        } else {
            #[allow(non_upper_case_globals)]
            match t {
                EnYacSQLType_YAC_SQLTYPE_QUERY => Ok(Executed::DQL(ResultSet(self.0))),
                EnYacSQLType_YAC_SQLTYPE_INSERT
                | EnYacSQLType_YAC_SQLTYPE_UPDATE
                | EnYacSQLType_YAC_SQLTYPE_DELETE
                | EnYacSQLType_YAC_SQLTYPE_MERGE => Ok(Executed::DML(Affection(self.0))),
                EnYacSQLType_YAC_SQLTYPE_GRANT
                | EnYacSQLType_YAC_SQLTYPE_REVOKE
                | EnYacSQLType_YAC_SQLTYPE_COMMIT
                | EnYacSQLType_YAC_SQLTYPE_ROLLBACK => Ok(Executed::DCL(Instruction(self.0))),
                _ => Ok(Executed::Unknown(self.0)),
            }
        }
    }

    pub fn result_set(self) -> ResultSet {
        ResultSet(self.0)
    }

    pub fn affection(self) -> Affection {
        Affection(self.0)
    }

    pub fn instruction(self) -> Instruction {
        Instruction(self.0)
    }
}

pub struct ResultSet(pub(crate) StatementHandle);

impl ResultSet {
    pub fn columns(&self) -> usize {
        let columms = 0;
        unsafe { yacNumResultCols(self.0 .0, &columms as *const _ as *mut _) };
        columms
    }

    pub fn iter_columns(&self) -> ColumnsIterator<'_> {
        ColumnsIterator::new(self)
    }

    pub fn rows(self) -> RowsIterator {
        RowsIterator::new(self)
    }
}

pub enum Executed {
    DQL(ResultSet),   // select
    DML(Affection),   // insert/update/delete/merge
    DCL(Instruction), // grant/revoke/commit/rollback
    Unknown(StatementHandle),
}

pub struct Affection(pub(crate) StatementHandle);

impl Affection {
    pub fn affected(&self) -> usize {
        let affected = 0usize;
        unsafe {
            yacGetStmtAttr(
                self.0 .0,
                EnYacStmtAttr_YAC_ATTR_ROWS_AFFECTED,
                &affected as *const _ as *mut _,
                std::mem::size_of_val(&affected) as _,
                null_mut(),
            );
        };
        affected
    }
}

pub struct Instruction(pub(crate) StatementHandle);
