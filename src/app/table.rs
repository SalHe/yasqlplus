use tabled::Tabled;
use yasqlplus_client::wrapper::Column;

pub struct ColumnWrapper<'a>(pub &'a Column);

impl Tabled for ColumnWrapper<'_> {
    const LENGTH: usize = 7;

    fn fields(&self) -> Vec<std::borrow::Cow<'_, str>> {
        vec![
            format!("{}", self.0.display_char_size).into(),
            self.0.name.clone().into(),
            format!(
                "{:?}{}",
                self.0.type_,
                if self.0.nullable { " ?" } else { "" }
            )
            .into(),
            format!("{}", self.0.precision).into(),
            format!("{}", self.0.scale).into(),
            format!("{}", self.0.char_size).into(),
            format!("{}", self.0.display_char_size).into(),
        ]
    }

    fn headers() -> Vec<std::borrow::Cow<'static, str>> {
        vec![
            "DisplaySize".into(),
            "Name".into(),
            "Type".into(),
            "Precision".into(),
            "Scale".into(),
            "CharSize".into(),
            "DisplayCharSize".into(),
        ]
    }
}
