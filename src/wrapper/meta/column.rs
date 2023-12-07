use std::{cell::Cell, ffi::CStr, ptr::null_mut};

use crate::{
    native::{
        yacColAttribute, EnYacColAttr_YAC_COL_ATTR_DISPLAY_SIZE, EnYacColAttr_YAC_COL_ATTR_NAME,
        EnYacColAttr_YAC_COL_ATTR_NULLABLE, EnYacColAttr_YAC_COL_ATTR_SIZE, EnYacResult_YAC_ERROR,
        YacColAttr,
    },
    wrapper::{Error, ResultSet, StatementHandle},
};

#[derive(Debug)]
pub struct Column {
    pub display_size: usize,
    pub size: usize,
    pub name: String,
    pub nullable: bool,
}

pub struct ColumnsIterator<'a> {
    curr: Cell<usize>,
    columns: usize,
    result_set: &'a ResultSet,
}

impl<'a> ColumnsIterator<'a> {
    pub(crate) fn new(result_set: &'a ResultSet) -> Result<Self, Error> {
        Ok(Self {
            curr: Cell::new(0),
            columns: result_set.columns()?,
            result_set,
        })
    }
}

impl<'a> Iterator for ColumnsIterator<'a> {
    type Item = Column;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr.get() >= self.columns {
            None
        } else {
            let col = self.curr.get();
            *self.curr.get_mut() = self.curr.get() + 1;

            let display_size: usize = get_column_attr(
                &self.result_set.0,
                EnYacColAttr_YAC_COL_ATTR_DISPLAY_SIZE,
                col,
            )
            .unwrap();
            let size: usize =
                get_column_attr(&self.result_set.0, EnYacColAttr_YAC_COL_ATTR_SIZE, col).unwrap();
            let nullable =
                get_column_attr(&self.result_set.0, EnYacColAttr_YAC_COL_ATTR_NULLABLE, col)
                    .unwrap();

            let name = [0u8; 4096];
            unsafe {
                yacColAttribute(
                    self.result_set.0 .0,
                    col as _,
                    EnYacColAttr_YAC_COL_ATTR_NAME,
                    name.as_ptr() as *mut _,
                    name.len() as _,
                    null_mut(),
                )
            };
            let name = CStr::from_bytes_until_nul(&name[..])
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();

            Some(Column {
                display_size,
                size,
                name,
                nullable,
            })
        }
    }
}

fn get_column_attr<T: Sized + Default>(
    stmt: &StatementHandle,
    attr: YacColAttr,
    column: usize,
) -> Result<T, Error> {
    let x = <T as Default>::default();
    let out_size = 0;
    if EnYacResult_YAC_ERROR
        == unsafe {
            yacColAttribute(
                stmt.0,
                column as _,
                attr,
                &x as *const _ as *mut _,
                std::mem::size_of::<T>() as _,
                &out_size as *const _ as *mut _,
            )
        }
    {
        Err(Error::get_yas_diag(None).unwrap())
    } else {
        Ok(x)
    }
}
