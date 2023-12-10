use crate::native::{yacFetch, EnYacResult_YAC_ERROR};

use super::{
    BigInt, Binder, Bool, Char, Column, Double, Float, Integer, NChar, NVarChar, Number, ResultSet,
    SmallInt, TinyInt, Type, Unsupported, Value, VarChar,
};

pub struct Row {}

pub struct RowsIterator {
    result_set: ResultSet,
    _columns: Vec<Column>,
    binders: Vec<Box<dyn Binder>>,
    fetched: usize,
}

impl RowsIterator {
    pub fn new(result_set: ResultSet) -> Self {
        let columns = result_set.iter_columns().collect::<Vec<_>>();
        let mut binders = columns
            .clone()
            .iter()
            .map(|x| -> Box<dyn Binder> {
                match x.type_ {
                    Type::Unknown => unreachable!(),
                    Type::Bool => Box::<Bool>::default(),
                    Type::TinyInt => Box::<TinyInt>::default(),
                    Type::SmallInt => Box::<SmallInt>::default(),
                    Type::Integer => Box::<Integer>::default(),
                    Type::BigInt => Box::<BigInt>::default(),
                    Type::Float => Box::<Float>::default(),
                    Type::Double => Box::<Double>::default(),
                    Type::Number => Box::<Number>::default(),
                    Type::Char => Box::<Char>::default(),
                    Type::NChar => Box::<NChar>::default(),
                    Type::VarChar => Box::<VarChar>::default(),
                    Type::NVarChar => Box::<NVarChar>::default(),
                    _ => Box::<Unsupported>::new(Unsupported),
                }
            })
            .collect::<Vec<_>>();

        binders
            .iter_mut()
            .enumerate()
            .for_each(|(col, binder)| unsafe { binder.bind_column(&result_set, col) });

        Self {
            _columns: columns,
            binders,
            fetched: 0,
            result_set,
        }
    }

    pub fn fetched(&self) -> usize {
        self.fetched
    }
}

impl Iterator for RowsIterator {
    type Item = Vec<Value>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut rows = self.fetched;
        if EnYacResult_YAC_ERROR
            == unsafe { yacFetch(self.result_set.0 .0, &mut rows as *const _ as *mut _) }
            || rows == 0
        {
            None
        } else {
            self.fetched += 1;
            Some(
                self.binders
                    .iter()
                    .map(|x| unsafe { x.get_data() })
                    .collect::<Vec<_>>(),
            )
        }
    }
}
