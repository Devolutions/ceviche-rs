#[macro_use]
#[cfg(windows)]
extern crate winapi;
extern crate widestring;

pub mod controller;

pub enum ServiceEvent {
        Stop
}