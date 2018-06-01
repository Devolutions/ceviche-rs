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

pub enum ServiceEvent {
        Stop
}