
use std::collections::HashMap;
use std::env;

use service::Service;

pub struct Factory {
    services: HashMap<String, Box<FnMut(Vec<String>) -> Box<Service>>>
}

impl Factory {
    pub fn new() -> Self {
        Factory {
            services: HashMap::new()
        }
    }

    pub fn register<F>(&mut self, name: &str, callback: F)
        where F: FnMut(Vec<String>) -> Box<Service> +'static {
        self.services.insert(name.to_string(), Box::new(callback));
    }

    pub fn call(&mut self, name: &str) -> Option<Box<Service>> {
        let args: Vec<String> = env::args().collect();
        if let Some(callback) = self.services.get_mut(name) {
            return Some((&mut *callback)(args));
        }
        None
    }
}
