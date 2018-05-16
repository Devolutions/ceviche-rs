extern crate libc;

#[macro_use]
#[cfg(windows)]
extern crate winapi;

use winapi::um::winsvc::*;
use winapi::um::winnt::*;
use winapi::um::winbase::*;
use winapi::um::libloaderapi::*;
use winapi::shared::minwindef::*;

use std::env;
use std::ptr;
use std::ffi::{OsStr,CString};
use std::os::windows::ffi::OsStrExt;
use std::iter::once;

use libc::{c_int, c_char, c_void};

#[allow(non_camel_case_types, non_snake_case)]

#[repr(C)]
STRUCT!{struct SERVICE_DESCRIPTION_W {
	lpDescription: LPWSTR,
}}
pub type PSERVICE_DESCRIPTION_W = *mut SERVICE_DESCRIPTION_W;

extern "system" {
	pub fn ChangeServiceConfig2W(
       hService: SC_HANDLE,
       dwInfoLevel: DWORD,
       lpInfo: LPVOID,
   ) -> BOOL;
}

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
    pub load_order_group: String,
    pub dependencies: String,
    pub account_name: String,
    pub password: String,
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
        	load_order_group: "".to_string(),
        	dependencies: "".to_string(),
        	account_name: "".to_string(),
        	password: "".to_string(),
        })
    }

    pub fn create(&mut self) -> bool {
    	unsafe {
			let filename = get_filename();
			let sc_manager = OpenSCManagerW(ptr::null_mut(), ptr::null_mut(), SC_MANAGER_ALL_ACCESS);

			if sc_manager.is_null() {
				return false;
			}

			let mut tag_id: DWORD = 0;

			let h_service = CreateServiceW(sc_manager,
				get_utf16(self.service_name.as_str()).as_ptr(),
				get_utf16(self.display_name.as_str()).as_ptr(),
				self.desired_access, self.service_type,
				self.start_type, self.error_control,
				get_utf16(filename.as_str()).as_ptr(),
				ptr::null_mut(),
				&mut tag_id,
				ptr::null_mut(), ptr::null_mut(), ptr::null_mut());

			if h_service.is_null() {
				return false;
			}

			let mut sd = SERVICE_DESCRIPTION_W {
				lpDescription: get_utf16(self.description.as_str()).as_mut_ptr(),
			};

			//let sd_ptr: *mut c_void = &mut sd;
			//let success = ChangeServiceConfig2W(ptr::null_mut(), 0, &*sd as *mut winapi::ctypes::c_void);

			CloseServiceHandle(sc_manager);

			true
		}
    }

	pub fn delete(&mut self) -> bool {
    	true
    }
}

impl Drop for Service {
	fn drop(&mut self) {
        unsafe {
            NtService_Free(self.ctx);
        }
    }
}

fn get_utf16(value : &str) -> Vec<u16> {
	OsStr::new(value).encode_wide().chain(once(0)).collect()
}

fn get_filename() -> String {
	unsafe {
		let mut filename: [u16; MAX_PATH] = [0; MAX_PATH];
		let nSize = GetModuleFileNameW(ptr::null_mut(),
			filename.as_mut_ptr(),
			filename.len() as DWORD);
		String::from_utf16(&filename).unwrap()
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
