/// The dummy controller is a mock controller, the only operation that as an
/// effect is calling it in standalone mode.
use Error;

use controller::ControllerInterface;

pub struct DummyController {}

impl ControllerInterface for DummyController {
    fn create(&mut self) -> Result<(), Error> {
        println!("Dummy controller: creating service (this has no effect on the system)");
        Ok(())
    }

    fn delete(&mut self) -> Result<(), Error> {
        println!("Dummy controller: deleting service (this has no effect on the system)");
        Ok(())
    }

    fn start(&mut self) -> Result<(), Error> {
        println!("Dummy controller: starting service (this has no effect on the system)");
        Ok(())
    }

    fn stop(&mut self) -> Result<(), Error> {
        println!("Dummy controller: stopping service (this has no effect on the system)");
        Ok(())
    }
}

impl DummyController {
    pub fn new(_service_name: &str, _display_name: &str, _description: &str) -> DummyController {
        DummyController {}
    }

    pub fn register(&mut self, _service_main_wrapper: fn()) -> Result<(), Error> {
        unimplemented!();
    }
}

#[macro_export]
macro_rules! Service {
    ($name:expr, $function:ident) => {
        fn service_main_wrapper() {
            ()
        }
    };
}
