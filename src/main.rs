extern crate libc;

use std::env;
use std::ffi::CString;
use libc::{c_int, c_char};

pub enum NtService {}

extern {
	pub fn NtService_ProcessCommandLine(ctx: *mut NtService, argc: c_int, argv: *const *const c_char) -> bool;
	pub fn NtService_New(serviceName: *const c_char, displayName: *const c_char, description: *const c_char) -> *mut NtService;
	pub fn NtService_Free(ctx: *mut NtService);
}

pub struct Service {
    ctx: *mut NtService
}

impl Service {
    pub fn new(service_name: &str, display_name: &str, description: &str) -> Option<Service> {
        let raw_service = unsafe {
			let service_name = CString::new(service_name).unwrap();
			let display_name = CString::new(display_name).unwrap();
			let description = CString::new(description).unwrap();
			NtService_New(service_name.as_ptr(), display_name.as_ptr(), description.as_ptr())
        };

        let args: Vec<CString> = env::args().filter_map(|arg| CString::new(arg).ok()).collect();
    	let c_args: Vec<*const c_char> = args.iter().map(|arg| arg.as_ptr()).collect();

    	unsafe {
    		NtService_ProcessCommandLine(raw_service, c_args.len() as c_int, c_args.as_ptr());
    	};

        Some(Service { ctx: raw_service })
    }
}

impl Drop for Service {
	fn drop(&mut self) {
        unsafe {
            NtService_Free(self.ctx);
        }
    }
}

fn main() {
	let service = Service::new("foobar", "FooBar Service", "This is the FooBar service");
}
