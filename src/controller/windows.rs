
use std::ptr;
use std::ffi::OsStr;
use std::iter::once;
use std::{thread, time};
use std::os::windows::ffi::OsStrExt;
use std::sync::mpsc;

use widestring::WideCString;
use winapi;
use winapi::um::winbase::*;
use winapi::um::winnt::*;
use winapi::um::winsvc::*;
use winapi::um::winuser::*;
use winapi::um::libloaderapi::*;
use winapi::um::errhandlingapi::*;
use winapi::shared::minwindef::*;
use winapi::shared::winerror::*;

use Error;
use ServiceEvent;
use controller::{ControllerInterface, ServiceMainFn};

static mut SERVICE_CONTROL_HANDLE: SERVICE_STATUS_HANDLE = ptr::null_mut();

STRUCT!{#[allow(non_snake_case)]
	struct SERVICE_DESCRIPTION_W {
    lpDescription: LPWSTR,
}}

extern "system" {
    pub fn ChangeServiceConfig2W(hService: SC_HANDLE, dwInfoLevel: DWORD, lpInfo: LPVOID) -> BOOL;
    pub fn StartServiceW(
        hService: SC_HANDLE,
        dwNumServiceArgs: DWORD,
        lpServiceArgVectors: *mut LPCWSTR,
    ) -> BOOL;
}

type WindowsServiceMainWrapperFn = extern "system" fn (argc: DWORD, argv: *mut LPWSTR);

struct Service {
    pub handle: SC_HANDLE
}

impl Drop for Service {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe{CloseServiceHandle(self.handle)};
        }
    }
}

struct ServiceControlManager {
    pub handle: SC_HANDLE
}

impl ServiceControlManager {
    fn open(desired_access: DWORD) -> Result<ServiceControlManager, Error> {
        let handle = unsafe { OpenSCManagerW(ptr::null_mut(), ptr::null_mut(), desired_access) };

        if handle.is_null() {
            Err(Error::new(&format!("OpenSCManager: {}", get_last_error_text())))
        } else {
            Ok(ServiceControlManager{handle})
        }
    }

    fn open_service(&self,  service_name: &str, desired_access: DWORD) -> Result<Service, Error> {
        let handle = unsafe {OpenServiceW(self.handle, get_utf16(service_name).as_ptr(), desired_access)};
        
        if handle.is_null() {
            Err(Error::new(&format!("OpenServiceW: {}", get_last_error_text())))
        } else {
            Ok(Service{handle})
        }
    }
}

impl Drop for ServiceControlManager {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe{CloseServiceHandle(self.handle)};
        }
    }
}

/// Manages the service on the system.
pub struct WindowsController {
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

impl ControllerInterface for WindowsController {
    fn create(&mut self) -> Result<(), Error> {
        unsafe {
            let service_manager = ServiceControlManager::open(SC_MANAGER_ALL_ACCESS)?;

            let filename = get_filename();
            let tag_id = 0;

            let service = CreateServiceW(
                service_manager.handle,
                get_utf16(self.service_name.as_str()).as_ptr(),
                get_utf16(self.display_name.as_str()).as_ptr(),
                self.desired_access,
                self.service_type,
                self.start_type,
                self.error_control,
                get_utf16(filename.as_str()).as_ptr(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
            );

            if service.is_null() {
                return Err(Error::new(&format!("CreateService: {}", get_last_error_text())));
            }

            self.tag_id = tag_id;

            let mut sd = SERVICE_DESCRIPTION_W {
                lpDescription: get_utf16(self.description.as_str()).as_mut_ptr(),
            };

            let p_sd = &mut sd as *mut _ as *mut winapi::ctypes::c_void;
            ChangeServiceConfig2W(service, SERVICE_CONFIG_DESCRIPTION, p_sd);
            CloseServiceHandle(service);

            Ok(())
        }
    }

    fn delete(&mut self) -> Result<(), Error> {
        unsafe {
            let service_manager = ServiceControlManager::open(SC_MANAGER_ALL_ACCESS)?;
            let service = service_manager.open_service(&self.service_name, SERVICE_ALL_ACCESS)?;

            if ControlService(service_manager.handle, SERVICE_CONTROL_STOP, &mut self.service_status) != 0 {
                while QueryServiceStatus(service.handle, &mut self.service_status) != 0 {
                    if self.service_status.dwCurrentState != SERVICE_STOP_PENDING {
                        break;
                    }
                    thread::sleep(time::Duration::from_millis(250));
                }
            }

            if DeleteService(service.handle) != 0 {
                return Err(Error::new(&format!("DeleteService: {}", get_last_error_text())));
            }

            Ok(())
        }
    }

    fn start(&mut self) -> Result<(), Error> {
        unsafe {
            let service_manager = ServiceControlManager::open(SC_MANAGER_ALL_ACCESS)?;
            let service = service_manager.open_service(&self.service_name, SERVICE_ALL_ACCESS)?;

            if StartServiceW(service.handle, 0, ptr::null_mut()) != 0 {
                while QueryServiceStatus(service.handle, &mut self.service_status) != 0 {
                    if self.service_status.dwCurrentState != SERVICE_START_PENDING {
                        break;
                    }
                    thread::sleep(time::Duration::from_millis(250));
                }
            }

            if self.service_status.dwCurrentState != SERVICE_RUNNING {
                return Err(Error::new("Failed to start service"));
            }

            Ok(())
        }
    }

    fn stop(&mut self) -> Result<(), Error> {
        unsafe {
            let service_manager = ServiceControlManager::open(SC_MANAGER_ALL_ACCESS)?;
            let service = service_manager.open_service(&self.service_name, SERVICE_ALL_ACCESS)?;

            if ControlService(service_manager.handle, SERVICE_CONTROL_STOP, &mut self.service_status) != 0 {
                while QueryServiceStatus(service.handle, &mut self.service_status) != 0 {
                    if self.service_status.dwCurrentState != SERVICE_STOP_PENDING {
                        break;
                    }
                    thread::sleep(time::Duration::from_millis(250));
                }
            } else {
                return Err(Error::new("ControlService: failed to stop service"));
            }

            if self.service_status.dwCurrentState != SERVICE_STOPPED {
                return Err(Error::new("Failed to stop service"));
            }

            Ok(())
        }
    }
}

impl WindowsController {
    pub fn new(
        service_name: &str,
        display_name: &str,
        description: &str,
    ) -> WindowsController {
        WindowsController {
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
                dwWaitHint: 0,
            },
            status_handle: ptr::null_mut(),
            controls_accepted: SERVICE_ACCEPT_STOP,
        }
    }

    /// Register the `service_main_wrapper` function, this function is generated by the `Service!` macro. 
    pub fn register(&mut self, service_main_wrapper: WindowsServiceMainWrapperFn) -> Result<(), Error> {
        unsafe {
            let service_name = get_utf16(self.service_name.as_str());

            let service_table: &[*const SERVICE_TABLE_ENTRYW] = &[
                &SERVICE_TABLE_ENTRYW {
                    lpServiceName: service_name.as_ptr(),
                    lpServiceProc: Some(service_main_wrapper),
                },
                ptr::null(),
            ];

            match StartServiceCtrlDispatcherW(*service_table.as_ptr()) {
                0 => Err(Error::new("StartServiceCtrlDispatcher")),
                _ => Ok(()),
            }
        }
    }
}

fn set_service_status(
    status_handle: SERVICE_STATUS_HANDLE,
    current_state: DWORD,
    wait_hint: DWORD,
) {
    let mut service_status = SERVICE_STATUS {
        dwServiceType: SERVICE_WIN32_OWN_PROCESS,
        dwCurrentState: current_state,
        dwControlsAccepted: SERVICE_ACCEPT_STOP | SERVICE_ACCEPT_SHUTDOWN | SERVICE_ACCEPT_PAUSE_CONTINUE |
                            SERVICE_ACCEPT_SESSIONCHANGE,
        dwWin32ExitCode: 0,
        dwServiceSpecificExitCode: 0,
        dwCheckPoint: 0,
        dwWaitHint: wait_hint,
    };
    unsafe {
        SetServiceStatus(status_handle, &mut service_status);
    }
}

unsafe extern "system" fn service_handler<T>(
    control: DWORD,
    event_type: DWORD,
    event_data: LPVOID,
    context: LPVOID,
) -> DWORD {
    let tx = context as *mut mpsc::Sender<ServiceEvent<T>>;

	match control {
		SERVICE_CONTROL_STOP | SERVICE_CONTROL_SHUTDOWN => {
            set_service_status(SERVICE_CONTROL_HANDLE, SERVICE_STOP_PENDING, 10);
            let _ = (*tx).send(ServiceEvent::Stop);
            return 0;
            },
        SERVICE_CONTROL_PAUSE => {
            let _ = (*tx).send(ServiceEvent::Pause);
            return 0;
        },
        SERVICE_CONTROL_CONTINUE => {
            let _ = (*tx).send(ServiceEvent::Continue);
            return 0;
        },
        SERVICE_CONTROL_SESSIONCHANGE => {
            let event = event_type as usize;
            let session_notification = event_data as PWTSSESSION_NOTIFICATION;
            let session_id = (*session_notification).dwSessionId;

            if event == WTS_CONSOLE_CONNECT || event == WTS_REMOTE_CONNECT {
                let _ = (*tx).send(ServiceEvent::SessionConnect(session_id));
                return 0;
            } else if event == WTS_CONSOLE_DISCONNECT || event == WTS_REMOTE_DISCONNECT {
                let _ = (*tx).send(ServiceEvent::SessionDisconnect(session_id));
                return 0;
            } else if event == WTS_SESSION_LOGON {
                let _ = (*tx).send(ServiceEvent::SessionLogon(session_id));
                return 0;
            } else if event == WTS_SESSION_LOGOFF {
                let _ = (*tx).send(ServiceEvent::SessionLogoff(session_id));
                return 0;
            } else if event == WTS_SESSION_LOCK {
                let _ = (*tx).send(ServiceEvent::SessionLock(session_id));
                return 0;
            } else if event == WTS_SESSION_UNLOCK {
                let _ = (*tx).send(ServiceEvent::SessionUnlock(session_id));
                return 0;
            } else {
                return 0;
            }
        },
        _ => return ERROR_CALL_NOT_IMPLEMENTED,
	};
}

fn get_args(argc: DWORD, argv: *mut LPWSTR) -> Vec<String> {
    let mut args = Vec::new();
    for i in 0..argc {
        unsafe {
            let s = *argv.offset(i as isize);
            let widestr = WideCString::from_ptr_str(s);
            args.push(widestr.to_string_lossy());
        }
    }
    args
}

pub fn get_utf16(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(once(0)).collect()
}

pub fn get_filename() -> String {
    unsafe {
        let mut filename = [0u16; MAX_PATH];
        let _size = GetModuleFileNameW(
            ptr::null_mut(),
            filename.as_mut_ptr(),
            filename.len() as DWORD,
        );
        String::from_utf16(&filename).unwrap_or_else(|_| String::from(""))
    }
}

pub fn get_username() -> String {
    unsafe {
        let mut size = 0;
        GetUserNameW(ptr::null_mut(), &mut size);
        let mut username = Vec::with_capacity(size as usize);
        GetUserNameW(username.as_mut_ptr(), &mut size);
        username.set_len(size as usize);
        String::from_utf16(&username).unwrap_or_else(|_| String::from(""))
    }
}

pub fn get_last_error_text() -> String {
    unsafe {
        let mut message = [0u16; 512];
        let length = FormatMessageW(
            FORMAT_MESSAGE_FROM_SYSTEM,
            ptr::null(),
            GetLastError(),
            0,
            message.as_mut_ptr(),
            message.len() as u32,
            ptr::null_mut(),
        );
        String::from_utf16(&message[0..length as usize]).unwrap_or_else(|_| String::from(""))
    }
}

/// Generates a `service_main_wrapper` that wraps the provided service main function.
#[macro_export]
macro_rules! Service { ( $name:expr, $function:ident ) => {
    extern crate winapi;
    use winapi::shared::minwindef::DWORD;
    use winapi::um::winnt::LPWSTR;

    extern "system" fn service_main_wrapper(argc: DWORD, argv: *mut LPWSTR) {
        dispatch($function, $name, argc, argv);
    }
}}

#[doc(hidden)]
pub fn dispatch<T>(service_main : ServiceMainFn<T>, name: &str, argc: DWORD, argv: *mut LPWSTR) {
    let args = get_args(argc, argv);
    let service_name = get_utf16(name);
    let (mut tx, rx) = mpsc::channel();
    let _tx = tx.clone();
    let ctrl_handle = unsafe {RegisterServiceCtrlHandlerExW(
        service_name.as_ptr(),
        Some(service_handler::<T>),
        &mut tx as *mut _ as LPVOID,
    )};
    unsafe{ SERVICE_CONTROL_HANDLE = ctrl_handle };
    set_service_status(ctrl_handle, SERVICE_START_PENDING, 0);
    set_service_status(ctrl_handle, SERVICE_RUNNING, 0);
    service_main(rx, _tx, args, false);
    set_service_status(ctrl_handle, SERVICE_STOPPED, 0);
}