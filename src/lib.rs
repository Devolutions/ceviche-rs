#[macro_use]
extern crate cfg_if;

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

impl Error {
    pub fn new(message: &str) -> Error {
        Error{message: String::from(message)}
    }
}

pub enum ServiceEvent {
        Stop
}