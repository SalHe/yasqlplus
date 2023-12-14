use std::{ffi::CStr, fmt::Display};

use crate::native::yacBindColumn;

use super::{ResultSet, Type};

pub trait Binder {
    // const TYPE: Type;

    /// # Safety
    /// TODO
    unsafe fn bind_column(&mut self, result_set: &ResultSet, column: usize);

    /// # Safety
    /// TODO
    unsafe fn get_data(&self) -> Option<Value>;
}

#[derive(Debug)]
pub enum Value {
    Bool(bool),
    TinyInt(i8),
    SmallInt(i16),
    Integer(i32),
    BigInt(i64),
    Float(f32),
    Double(f64),
    Number(String), // TODO typed
    Bit(u64),
    Char(String),
    VarChar(String),
    NChar(String),
    NVarChar(String),
    Date(String),        // TODO typed
    Time(String),        // TODO typed
    Unsupported(String), // TODO
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(v) => v.fmt(f),
            Value::TinyInt(v) => v.fmt(f),
            Value::SmallInt(v) => v.fmt(f),
            Value::Integer(v) => v.fmt(f),
            Value::BigInt(v) => v.fmt(f),
            Value::Float(v) => v.fmt(f),
            Value::Double(v) => v.fmt(f),
            Value::Number(v) => v.fmt(f),
            Value::Bit(v) => v.fmt(f),
            Value::Char(v) => v.fmt(f),
            Value::VarChar(v) => v.fmt(f),
            Value::NChar(v) => v.fmt(f),
            Value::NVarChar(v) => v.fmt(f),
            Value::Date(v) => v.fmt(f),
            Value::Time(v) => v.fmt(f),
            Value::Unsupported(v) => v.fmt(f),
        }
    }
}

macro_rules! sized_value {
    ($type:ident: $ty:ty => $sql_ty:expr) => {
        #[derive(Default)]
        pub struct $type($ty, i32);

        impl Binder for $type {
            // const TYPE: Type = $sql_ty;
            unsafe fn bind_column(&mut self, result_set: &ResultSet, column: usize) {
                unsafe {
                    yacBindColumn(
                        result_set.0 .0,
                        column as _,
                        std::mem::transmute($sql_ty),
                        &self.0 as *const _ as *mut _,
                        std::mem::size_of_val(&self.0) as _,
                        &self.1 as *const _ as *mut _,
                    )
                };
            }

            unsafe fn get_data(&self) -> Option<Value> {
                if self.1 == -1 {
                    None
                } else {
                    Some(Value::$type(self.0.clone()))
                }
            }
        }
    };
}

sized_value! { Bool: bool           => Type::Bool}
sized_value! { TinyInt: i8          => Type::TinyInt}
sized_value! { SmallInt: i16        => Type::SmallInt}
sized_value! { Integer: i32         => Type::Integer}
sized_value! { BigInt: i64          => Type::BigInt}
sized_value! { Float: f32           => Type::Float}
sized_value! { Double: f64          => Type::Double}
sized_value! { Bit: u64             => Type::Bit}

macro_rules! string_value {
    ($type:ident @ $buffer_len:expr => $sql_ty:expr) => {
        pub struct $type([u8; $buffer_len], i32);

        impl Default for $type {
            fn default() -> Self {
                Self([0; $buffer_len], 0)
            }
        }

        impl Binder for $type {
            // const TYPE: Type = $sql_ty;
            unsafe fn bind_column(&mut self, result_set: &ResultSet, column: usize) {
                unsafe {
                    yacBindColumn(
                        result_set.0 .0,
                        column as _,
                        std::mem::transmute($sql_ty),
                        &self.0 as *const _ as *mut _,
                        self.0.len() as _,
                        &self.1 as *const _ as *mut _,
                    )
                };
            }

            unsafe fn get_data(&self) -> Option<Value> {
                if self.1 == -1 {
                    None
                } else {
                    Some(Value::$type(
                        CStr::from_bytes_until_nul(&self.0)
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_string(),
                    ))
                }
            }
        }
    };
}

string_value! {Char            @ 4096 => Type::Char}
string_value! {NChar           @ 4096 => Type::NChar}
string_value! {VarChar         @ 4096 => Type::VarChar}
string_value! {NVarChar        @ 4096 => Type::NVarChar}
string_value! {Date            @ 4096 => Type::VarChar}
string_value! {Number          @ 4096 => Type::VarChar}
string_value! {Time            @ 4096 => Type::VarChar}
string_value! {Unsupported     @ 4096 => Type::VarChar}
