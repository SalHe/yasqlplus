use crate::native::{yacNumResultCols, EnYacResult_YAC_ERROR};

use super::{ColumnsIterator, Error, StatementHandle};

pub struct ResultSet(pub(crate) StatementHandle);

impl ResultSet {
    pub(crate) fn new(stmt: StatementHandle) -> Self {
        ResultSet(stmt)
    }

    pub fn columns(&self) -> Result<usize, Error> {
        let columms = 0;
        if EnYacResult_YAC_ERROR
            == unsafe { yacNumResultCols(self.0 .0, &columms as *const _ as *mut _) }
        {
            return Err(Error::get_yas_diag(None).unwrap());
        }
        Ok(columms as _)
    }

    pub fn iter_columns(&self) -> Result<ColumnsIterator<'_>, Error> {
        ColumnsIterator::new(self)
    }
}
