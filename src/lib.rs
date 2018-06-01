#[macro_use]
extern crate cfg_if;

use std::fmt;

cfg_if!{
    if #[cfg(windows)] {
        #[macro_use]
        extern crate winapi;
        extern crate widestring;
    }
}

pub mod controller;

#[derive(Debug)]
pub struct Error {
    pub message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.message,)
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        &self.message
    }
}

impl Error {
    pub fn new(message: &str) -> Error {
        Error{message: String::from(message)}
    }
}

pub enum ServiceEvent {
        Stop
}