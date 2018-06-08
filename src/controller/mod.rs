use std::sync::mpsc;

use Error;
use ServiceEvent;

// On platforms other than Windows we default to the dummy controller.
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

/// Signature of the service main function.
/// `rx` receives the events that are sent to the service. `args` is the list or arguments that were passed to
/// the service. When `standalone_mode` is true, the service main function is being called directly (outside of
/// the system service support).
pub type ServiceMainFn =
    fn(rx: mpsc::Receiver<ServiceEvent>, args: Vec<String>, standalone_mode: bool) -> u32;

/// Controllers implement this interface. They also need to implement the `register()` method; because the signature
/// of service_main_wrapper depends on the system the method is not part of the interface.
pub trait ControllerInterface {
    /// Creates the service on the system.
    fn create(&mut self) -> Result<(), Error>;
    /// Deletes the service.
    fn delete(&mut self) -> Result<(), Error>;
    /// Starts the service.
    fn start(&mut self) -> Result<(), Error>;
    /// Stops the service.
    fn stop(&mut self) -> Result<(), Error>;
}
