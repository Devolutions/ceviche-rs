
use std::sync::mpsc;

use Error;
use ServiceEvent;

cfg_if!{
    if #[cfg(windows)] {
        #[macro_use]
        mod windows;
        pub use controller::windows::WindowsController as Controller;
        pub use controller::windows::dispatch;
    } else {
        mod dummy;
        pub use controller::dummy::DummyController as Controller;
    }
}

pub type ServiceMainFn = fn(mpsc::Receiver<ServiceEvent>, Vec<String>, bool) -> u32;

pub trait ControllerInterface {
    fn create(&mut self) -> Result<(), Error>;
    fn delete(&mut self) -> Result<(), Error>;
    fn start(&mut self) -> Result<(), Error>;
    fn stop(&mut self) -> Result<(), Error>;
}