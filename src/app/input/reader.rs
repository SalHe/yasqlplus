use std::{
    cell::RefCell,
    io::{BufRead, BufReader, Read},
};

use crate::command::{parse_command, Command, ParseError};

use super::{Input, InputError, InputSettings};

pub struct BufReaderInput<R: ?Sized + Read> {
    reader: RefCell<BufReader<R>>,
}

impl<R: Read> BufReaderInput<R> {
    // pub fn new(reader: BufReader<R>) -> Self {
    //     Self {
    //         reader: RefCell::new(reader),
    //     }
    // }
}

impl<R: Read> InputSettings for BufReaderInput<R> {
    fn need_echo(&self) -> bool {
        true
    }
}

impl<R: Read> Input for BufReaderInput<R> {
    fn get_command(&self) -> Result<Option<(Command, String)>, InputError> {
        let mut command_string = String::new();
        loop {
            if 0 == self.reader.borrow_mut().read_line(&mut command_string)? {
                return Err(InputError::Eof);
            }
            let to_parse = command_string.trim_end_matches(['\r', '\n']);
            match parse_command(to_parse) {
                Ok(command) => return Ok(Some((command, to_parse.to_string()))),
                Err(ParseError::Incomplete(_)) => {}
                Err(err) => return Err(err.into()),
            }
        }
    }

    fn line(&self, _prompt: &str) -> Result<String, InputError> {
        let mut line = String::new();
        self.reader.borrow_mut().read_line(&mut line)?;
        Ok(line)
    }
}
