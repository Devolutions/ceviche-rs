extern crate libc;

#[cfg(windows)]
extern crate winapi;

use winapi::um::winsvc::*;
use winapi::um::winnt::*;
use winapi::um::libloaderapi::*;
use winapi::um::winbase::*;
use winapi::shared::minwindef::{DWORD, MAX_PATH};

use std::env;
use std::ptr;
use std::ffi::CString;

use libc::{c_int, c_char};

pub enum NtService {}

extern {
	pub fn NtService_ProcessCommandLine(ctx: *mut NtService, argc: c_int, argv: *const *const c_char) -> bool;
	pub fn NtService_New(serviceName: *const c_char, displayName: *const c_char, description: *const c_char) -> *mut NtService;
	pub fn NtService_Free(ctx: *mut NtService);
}

pub struct Service {
    ctx: *mut NtService,
    pub service_name: String,
    pub display_name: String,
    pub description: String,
    pub desired_access: DWORD,
    pub service_type: DWORD,
    pub start_type: DWORD,
    pub error_control: DWORD,
    pub tag_id: DWORD,
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

        Some(Service {
        	ctx: raw_service,
        	service_name: service_name.to_string(),
        	display_name: display_name.to_string(),
        	description: description.to_string(),
        	desired_access: SERVICE_ALL_ACCESS,
        	service_type: SERVICE_WIN32_OWN_PROCESS,
        	start_type: SERVICE_AUTO_START,
        	error_control: SERVICE_ERROR_NORMAL,
        	tag_id: 0,
        })
    }

    pub fn create(&mut self) {

    }
}

impl Drop for Service {
	fn drop(&mut self) {
        unsafe {
            NtService_Free(self.ctx);
        }
    }
}

#[allow(non_camel_case_types, non_snake_case)]

fn get_filename() -> String {
	unsafe {
		let mut filename: [u16; MAX_PATH] = [0; MAX_PATH];
		let nSize = GetModuleFileNameW(ptr::null_mut(),
			filename.as_ptr() as *mut u16,
			filename.len() as DWORD);
		String::from_utf16(&lpFilename).unwrap()
	}
}

fn get_username() -> String {
    unsafe {
        let mut size = 0;
        GetUserNameW(ptr::null_mut(), &mut size);
        let mut username = Vec::with_capacity(size as usize);
        GetUserNameW(username.as_mut_ptr(), &mut size);
        username.set_len(size as usize);
        String::from_utf16(&username).unwrap()
    }
}

fn main() {
	let username = get_username();
	let filename = get_filename();
	println!("username: {} filename: {}", username, filename);
	let service = Service::new("foobar", "FooBar Service", "This is the FooBar service");
}
