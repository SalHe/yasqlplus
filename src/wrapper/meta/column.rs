use std::{cell::Cell, ffi::CStr, ptr::null_mut};

use crate::{
    native::{
        yacColAttribute, EnYacColAttr_YAC_COL_ATTR_CHAR_SIZE,
        EnYacColAttr_YAC_COL_ATTR_DISPLAY_CHAR_SIZE, EnYacColAttr_YAC_COL_ATTR_DISPLAY_SIZE,
        EnYacColAttr_YAC_COL_ATTR_NAME, EnYacColAttr_YAC_COL_ATTR_NULLABLE,
        EnYacColAttr_YAC_COL_ATTR_PRECISION, EnYacColAttr_YAC_COL_ATTR_SCALE,
        EnYacColAttr_YAC_COL_ATTR_SIZE, EnYacColAttr_YAC_COL_ATTR_TYPE, EnYacResult_YAC_ERROR,
        YacColAttr,
    },
    wrapper::{Error, ResultSet, StatementHandle},
};

#[repr(u32)]
#[derive(Debug)]
pub enum Type {
    Unknown = 0,
    Bool = 1,
    TinyInt = 2,
    SmallInt = 3,
    Integer = 4,
    BigInt = 5,
    Float = 10,
    Double = 11,
    Number = 12,
    Date = 13,
    ShortTime = 15,
    Timestamp = 16,
    YmInterval = 19,
    DsInterval = 20,
    Char = 24,
    NChar = 25,
    VarChar = 26,
    NVarChar = 27,
    Binary = 28,
    Clob = 29,
    Blob = 30,
    Bit = 31,
    RowId = 32,
    NClob = 33,
    Cursor = 34,
    Json = 35,
}

impl Default for Type {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug)]
pub struct Column {
    pub display_size: usize,
    pub name: String,
    pub size: usize,
    pub type_: Type,
    pub nullable: bool,
    pub precision: usize,
    pub scale: usize,
    pub char_size: usize,
    pub display_char_size: usize,
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
            let type_ =
                get_column_attr(&self.result_set.0, EnYacColAttr_YAC_COL_ATTR_TYPE, col).unwrap();
            let precision =
                get_column_attr(&self.result_set.0, EnYacColAttr_YAC_COL_ATTR_PRECISION, col)
                    .unwrap();
            let scale =
                get_column_attr(&self.result_set.0, EnYacColAttr_YAC_COL_ATTR_SCALE, col).unwrap();
            let char_size =
                get_column_attr(&self.result_set.0, EnYacColAttr_YAC_COL_ATTR_CHAR_SIZE, col)
                    .unwrap();
            let display_char_size = get_column_attr(
                &self.result_set.0,
                EnYacColAttr_YAC_COL_ATTR_DISPLAY_CHAR_SIZE,
                col,
            )
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
                type_,
                nullable,
                precision,
                scale,
                char_size,
                display_char_size,
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
