//! ceviche is a wrapper to write a service/daemon.
//!
//! At the moment only Windows services are supported. The Service macro is inspired
//! from the [winservice](https://crates.io/crates/winservice) crate.
//!
//! A service implements a service main function and is generated by invoking
//! the `Service!` macro. The events are sent to the service over the `rx` channel.
//!
//! ```rust
//! fn my_service_main(rx: mpsc::Receiver<ServiceEvent>, args: Vec<String>, standalone_mode: bool) -> u32 {
//!    loop {
//!        if let Ok(control_code) = rx.recv() {
//!            match control_code {
//!                ServiceEvent::Stop => break,
//!                _ => (),
//!            }
//!        }
//!    }
//!    0
//! }
//!
//! Service!("Foobar", my_service_main);
//! ```
//!
//! The Controller is a helper to create, remove, start or stop the service
//! on the system. ceviche also supports a standalone mode were the service
//! code runs as a normal executable which can be useful for development and
//! debugging.
//!
//! ```rust
//! static SERVICE_NAME: &'static str = "foobar";
//! static DISPLAY_NAME: &'static str = "FooBar Service";
//! static DESCRIPTION: &'static str = "This is the FooBar service";
//!
//! fn main() {
//!     let yaml = load_yaml!("cli.yml");
//!     let app = App::from_yaml(yaml);
//!     let matches = app.version(crate_version!()).get_matches();
//!     let cmd = matches.value_of("cmd").unwrap_or("").to_string();
//!
//!     let mut controller = Controller::new(SERVICE_NAME, DISPLAY_NAME, DESCRIPTION);
//!
//!     match cmd.as_str() {
//!         "create" => controller.create(),
//!         "delete" => controller.delete(),
//!         "start" => controller.start(),
//!         "stop" => controller.stop(),
//!         "standalone" => {
//!             let (tx, rx) = mpsc::channel();
//!
//!             ctrlc::set_handler(move || {
//!                 let _ = tx.send(ServiceEvent::Stop);
//!             }).expect("Failed to register Ctrl-C handler");
//!
//!             my_service_main(rx, vec![], true);
//!         }
//!         _ => {
//!             let _result = controller.register(service_main_wrapper);
//!         }
//!     }
//! }
//!
//! ```

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

/// Manages the service on the system.
pub mod controller;

/// Service errors
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
        Error {
            message: String::from(message),
        }
    }
}

/// Events that are sent to the service.
pub enum ServiceEvent {
    Continue,
    Pause,
    Stop,
    SessionConnect(u32),
    SessionDisconnect(u32),
    SessionLock(u32),
    SessionUnlock(u32),
}

impl fmt::Display for ServiceEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            ServiceEvent::Continue => write!(f, "Continue"),
            ServiceEvent::Pause => write!(f, "Pause"),
            ServiceEvent::Stop => write!(f, "Stop"),
            ServiceEvent::SessionConnect(id) => write!(f, "SessionConnect({})", id),
            ServiceEvent::SessionDisconnect(id) => write!(f, "SessionDisconnect({})", id),
            ServiceEvent::SessionLock(id) => write!(f, "SessionLock({})", id),
            ServiceEvent::SessionUnlock(id) => write!(f, "SessionUnlock({})", id),
        }
    }
}