extern crate libc;

#[macro_use]
extern crate clap;

#[macro_use]
#[cfg(windows)]
extern crate winapi;

use winapi::um::winsvc::*;
use winapi::um::winnt::*;
use winapi::um::winbase::*;
use winapi::um::libloaderapi::*;
use winapi::um::errhandlingapi::*;
use winapi::shared::minwindef::*;

use std::env;
use std::ptr;
use std::ffi::{OsStr,CString};
use std::os::windows::ffi::OsStrExt;
use std::iter::once;
use std::{thread, time};
use std::io::{Error, ErrorKind};

use libc::{c_int, c_char};

use clap::App;

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
    pub fn StartServiceW(
		hService: SC_HANDLE,
		dwNumServiceArgs: DWORD,
		lpServiceArgVectors: *mut LPCWSTR,
	) -> BOOL;
}

pub struct Service {
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
    pub service_status: SERVICE_STATUS,
    pub status_handle: SERVICE_STATUS_HANDLE,
    pub controls_accepted: DWORD,
}

impl Service {
    pub fn new(service_name: &str, display_name: &str, description: &str) -> Option<Service> {
        Some(Service {
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
        	service_status: SERVICE_STATUS {
				dwServiceType: SERVICE_WIN32_OWN_PROCESS,
				dwCurrentState: SERVICE_STOPPED,
				dwControlsAccepted: 0,
				dwWin32ExitCode: 0,
				dwServiceSpecificExitCode: 0,
				dwCheckPoint: 0,
				dwWaitHint: 0
			},
			status_handle: ptr::null_mut(),
			controls_accepted: SERVICE_ACCEPT_STOP,
        })
    }

    pub fn create(&mut self) -> bool {
    	unsafe {
			let filename = get_filename();
			let sc_manager = OpenSCManagerW(ptr::null_mut(), ptr::null_mut(), SC_MANAGER_ALL_ACCESS);

			if sc_manager.is_null() {
				print!("OpenSCManager: {}", get_last_error_text());
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
				ptr::null_mut(), //&mut tag_id,
				ptr::null_mut(), ptr::null_mut(), ptr::null_mut());

			if h_service.is_null() {
				print!("CreateService: {}", get_last_error_text());
				return false;
			}

			self.tag_id = tag_id;

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
		unsafe {
			let sc_manager = OpenSCManagerW(ptr::null_mut(), ptr::null_mut(), SC_MANAGER_ALL_ACCESS);

			if sc_manager.is_null() {
				print!("OpenSCManager: {}", get_last_error_text());
				return false;
			}

			let h_service = OpenServiceW(sc_manager,
				get_utf16(self.service_name.as_str()).as_ptr(), SERVICE_ALL_ACCESS);

			if h_service.is_null() {
				print!("OpenService: {}", get_last_error_text());
				return false;
			}

			if ControlService(sc_manager, SERVICE_CONTROL_STOP, &mut self.service_status) != 0 {
				while QueryServiceStatus(h_service, &mut self.service_status) != 0 {
					if self.service_status.dwCurrentState != SERVICE_STOP_PENDING {
						break;
					}
					thread::sleep(time::Duration::from_millis(250));
				}
			} else {
				return false;
			}

			if DeleteService(h_service) != 0 {
				print!("DeleteService: {}", get_last_error_text());
			}

			CloseServiceHandle(h_service);
			CloseServiceHandle(sc_manager);

	    	true
	    }
    }

    pub fn start(&mut self) -> bool {
		unsafe {
			let sc_manager = OpenSCManagerW(ptr::null_mut(), ptr::null_mut(), SC_MANAGER_ALL_ACCESS);

			if sc_manager.is_null() {
				print!("OpenSCManager: {}", get_last_error_text());
				return false;
			}

			let h_service = OpenServiceW(sc_manager,
				get_utf16(self.service_name.as_str()).as_ptr(), SERVICE_ALL_ACCESS);

			if h_service.is_null() {
				print!("OpenService: {}", get_last_error_text());
				return false;
			}

			if StartServiceW(h_service, 0, ptr::null_mut()) != 0 {
				while QueryServiceStatus(h_service, &mut self.service_status) != 0 {
					if self.service_status.dwCurrentState != SERVICE_START_PENDING {
						break;
					}
					thread::sleep(time::Duration::from_millis(250));
				}
			}

			if self.service_status.dwCurrentState != SERVICE_RUNNING {
				println!("failed to start service");
				return false;
			}

			CloseServiceHandle(h_service);
			CloseServiceHandle(sc_manager);

	    	true
	    }
    }

    pub fn stop(&mut self) -> bool {
		unsafe {
			let sc_manager = OpenSCManagerW(ptr::null_mut(), ptr::null_mut(), SC_MANAGER_ALL_ACCESS);

			if sc_manager.is_null() {
				print!("OpenSCManager: {}", get_last_error_text());
				return false;
			}

			let h_service = OpenServiceW(sc_manager,
				get_utf16(self.service_name.as_str()).as_ptr(), SERVICE_ALL_ACCESS);

			if h_service.is_null() {
				print!("OpenService: {}", get_last_error_text());
				return false;
			}

			if ControlService(sc_manager, SERVICE_CONTROL_STOP, &mut self.service_status) != 0 {
				while QueryServiceStatus(h_service, &mut self.service_status) != 0 {
					if self.service_status.dwCurrentState != SERVICE_STOP_PENDING {
						break;
					}
					thread::sleep(time::Duration::from_millis(250));
				}
			} else {
				println!("failed to stop service");
				return false;
			}

			if self.service_status.dwCurrentState != SERVICE_STOPPED {
				println!("failed to stop service");
				return false;	
			}

			CloseServiceHandle(h_service);
			CloseServiceHandle(sc_manager);

	    	true
	    }
    }

    pub fn register(&mut self) -> Result<(), Error>
	{
		unsafe {
			let service_name = get_utf16(self.service_name.as_str());

			let service_table: &[*const SERVICE_TABLE_ENTRYW] = &[
				&SERVICE_TABLE_ENTRYW {
					lpServiceName: service_name.as_ptr(),
					lpServiceProc: Some(service_main),
				},
				ptr::null()
			];

			match StartServiceCtrlDispatcherW(*service_table.as_ptr())
			{
				0 => Err(Error::new(ErrorKind::Other, "StartServiceCtrlDispatcher")),
				_ => Ok(())
			}
		}
	}
}

impl Drop for Service {
	fn drop(&mut self) {
        unsafe {
        }
    }
}

fn set_service_status(status_handle: SERVICE_STATUS_HANDLE, current_state: DWORD, wait_hint: DWORD) {
	let mut service_status = SERVICE_STATUS {
		dwServiceType: SERVICE_WIN32_OWN_PROCESS,
		dwCurrentState: current_state,
		dwControlsAccepted: SERVICE_ACCEPT_STOP | SERVICE_ACCEPT_SHUTDOWN,
		dwWin32ExitCode: 0,
		dwServiceSpecificExitCode: 0,
		dwCheckPoint: 0,
		dwWaitHint: wait_hint,
	};
	unsafe {
		SetServiceStatus(status_handle, &mut service_status);
	}
}

unsafe extern "system"
fn service_main(argc: DWORD, argv: *mut LPWSTR) {
	let service_name = get_utf16("foobar");
	let ctrl_handle = RegisterServiceCtrlHandlerExW(service_name.as_ptr(), Some(service_handler), ptr::null_mut());
	set_service_status(ctrl_handle, SERVICE_START_PENDING, 0);
	set_service_status(ctrl_handle, SERVICE_RUNNING, 0);
	thread::sleep(time::Duration::from_millis(5000)); // "run" for 5 seconds
	set_service_status(ctrl_handle, SERVICE_STOPPED, 0);
}

unsafe extern "system"
fn service_handler(control: DWORD, event_type: DWORD, event_data: LPVOID, context: LPVOID) -> DWORD {
	match control {
		SERVICE_CONTROL_STOP | SERVICE_CONTROL_SHUTDOWN => {
			//set_service_status(ctrl_handle, SERVICE_STOP_PENDING, 0);
		}
		_ => {}
	};
	0
}

pub fn get_utf16(value : &str) -> Vec<u16> {
	OsStr::new(value).encode_wide().chain(once(0)).collect()
}

pub fn get_filename() -> String {
	unsafe {
		let mut filename: [u16; MAX_PATH] = [0; MAX_PATH];
		let nSize = GetModuleFileNameW(ptr::null_mut(),
			filename.as_mut_ptr(),
			filename.len() as DWORD);
		String::from_utf16(&filename).unwrap()
	}
}

pub fn get_username() -> String {
    unsafe {
        let mut size = 0;
        GetUserNameW(ptr::null_mut(), &mut size);
        let mut username = Vec::with_capacity(size as usize);
        GetUserNameW(username.as_mut_ptr(), &mut size);
        username.set_len(size as usize);
        String::from_utf16(&username).unwrap()
    }
}

pub fn get_last_error_text() -> String {
	unsafe {
		let mut message = [0u16; 512];
		let length = FormatMessageW(FORMAT_MESSAGE_FROM_SYSTEM, ptr::null(), GetLastError(),
			0, message.as_mut_ptr(), message.len() as u32, ptr::null_mut());
		String::from_utf16(&message[0..length as usize]).unwrap()
	}
}
