use std::{fs::File, io::Stdout};

pub trait OutputSettings {
    fn is_terminal(&self) -> bool;
}

impl OutputSettings for Stdout {
    fn is_terminal(&self) -> bool {
        console::Term::stdout().is_term()
    }
}

impl OutputSettings for File {
    fn is_terminal(&self) -> bool {
        false
    }
}

pub trait Output: OutputSettings + std::io::Write {}

impl Output for Stdout {}
impl Output for File {}
