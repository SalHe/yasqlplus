use crate::command::Command;

#[derive(Default)]
pub struct States {
    pub command: Option<Command>,
}
