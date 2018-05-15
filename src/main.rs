extern crate libc;

use std::env;
use std::ffi::CString;
use libc::{c_int, c_char};

pub enum NtService {}

extern "C" {
	pub fn NtService_ProcessCommandLine(ctx: *mut NtService, argc: c_int, argv: *const *const c_char) -> bool;
	pub fn NtService_New(serviceName: *const c_char, displayName: *const c_char, description: *const c_char) -> *mut NtService;
	pub fn NtService_Free(ctx: *mut NtService);
}

extern "C" {
    fn main_c(argc: c_int, argv: *const *const c_char) -> c_int;
}

fn main_rs() {
	unsafe {
		let service_name = CString::new("foobar").unwrap();
		let display_name = CString::new("FooBar Service").unwrap();
		let description = CString::new("This is the FooBar service").unwrap();
		let service = NtService_New(service_name.as_ptr(), display_name.as_ptr(), description.as_ptr());

    	let args: Vec<CString> = env::args().filter_map(|arg| CString::new(arg).ok()).collect();
    	let c_args: Vec<*const c_char> = args.iter().map(|arg| arg.as_ptr()).collect();

    	NtService_ProcessCommandLine(service, c_args.len() as c_int, c_args.as_ptr());

		NtService_Free(service);
	};
}

fn main() {
	main_rs();
	return;

    let args: Vec<CString> = env::args().filter_map(|arg| CString::new(arg).ok()).collect();
    let c_args: Vec<*const c_char> = args.iter().map(|arg| arg.as_ptr()).collect();

    unsafe {
        main_c(c_args.len() as c_int, c_args.as_ptr());
    };
}
