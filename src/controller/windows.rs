use std::ffi::{c_void, OsStr};
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{mpsc, Mutex, PoisonError};
use std::{thread, time};

use widestring::WideCString;
use windows_sys::core::PWSTR;
use windows_sys::Win32::{
    Foundation::{
        GetLastError, ERROR_CALL_NOT_IMPLEMENTED, ERROR_SERVICE_SPECIFIC_ERROR, MAX_PATH,
    },
    Security::SC_HANDLE,
    System::{
        Diagnostics::Debug::{FormatMessageW, FORMAT_MESSAGE_FROM_SYSTEM},
        LibraryLoader::GetModuleFileNameW,
        RemoteDesktop::WTSSESSION_NOTIFICATION,
        Services::*,
    },
    UI::WindowsAndMessaging::{
        WTS_CONSOLE_CONNECT, WTS_CONSOLE_DISCONNECT, WTS_REMOTE_CONNECT, WTS_REMOTE_DISCONNECT,
        WTS_SESSION_LOCK, WTS_SESSION_LOGOFF, WTS_SESSION_LOGON, WTS_SESSION_UNLOCK,
    },
};

use crate::controller::{ControllerInterface, ServiceMainFn};
use crate::session;
use crate::Error;
use crate::ServiceEvent;

type DWORD = u32;
type LPVOID = *mut c_void;

/// Wait hint reported with `SERVICE_START_PENDING`, in milliseconds.
const START_PENDING_WAIT_HINT_MS: DWORD = 5_000;
/// Wait hint reported with `SERVICE_STOP_PENDING`, in milliseconds.
const STOP_PENDING_WAIT_HINT_MS: DWORD = 30_000;

/// Exit code returned by the service main function, read by `register` once the dispatcher returns.
static SERVICE_EXIT_CODE: AtomicU8 = AtomicU8::new(0);

type WindowsServiceMainWrapperFn = extern "system" fn(argc: DWORD, argv: *mut PWSTR);
pub type Session = session::Session_<u32>;

struct Service {
    pub handle: SC_HANDLE,
}

impl Drop for Service {
    fn drop(&mut self) {
        if !(self.handle == 0) {
            unsafe { CloseServiceHandle(self.handle) };
        }
    }
}

struct ServiceControlManager {
    pub handle: SC_HANDLE,
}

impl ServiceControlManager {
    fn open(desired_access: DWORD) -> Result<ServiceControlManager, Error> {
        let handle = unsafe { OpenSCManagerW(ptr::null_mut(), ptr::null_mut(), desired_access) };

        if handle == 0 {
            Err(Error::new(&format!(
                "OpenSCManager: {}",
                get_last_error_text()
            )))
        } else {
            Ok(ServiceControlManager { handle })
        }
    }

    fn open_service(&self, service_name: &str, desired_access: DWORD) -> Result<Service, Error> {
        let handle = unsafe {
            OpenServiceW(
                self.handle,
                get_utf16(service_name).as_ptr(),
                desired_access,
            )
        };

        if handle == 0 {
            Err(Error::new(&format!(
                "OpenServiceW: {}",
                get_last_error_text()
            )))
        } else {
            Ok(Service { handle })
        }
    }
}

impl Drop for ServiceControlManager {
    fn drop(&mut self) {
        if !(self.handle == 0) {
            unsafe { CloseServiceHandle(self.handle) };
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

            if service == 0 {
                return Err(Error::new(&format!(
                    "CreateService: {}",
                    get_last_error_text()
                )));
            }

            self.tag_id = tag_id;

            let mut description = get_utf16(self.description.as_str());

            let mut sd = SERVICE_DESCRIPTIONW {
                lpDescription: description.as_mut_ptr(),
            };

            let p_sd = &mut sd as *mut _ as *mut c_void;
            ChangeServiceConfig2W(service, SERVICE_CONFIG_DESCRIPTION, p_sd);
            CloseServiceHandle(service);

            Ok(())
        }
    }

    fn delete(&mut self) -> Result<(), Error> {
        unsafe {
            let service_manager = ServiceControlManager::open(SC_MANAGER_ALL_ACCESS)?;
            let service = service_manager.open_service(&self.service_name, SERVICE_ALL_ACCESS)?;

            if ControlService(
                service.handle,
                SERVICE_CONTROL_STOP,
                &mut self.service_status,
            ) != 0
            {
                while QueryServiceStatus(service.handle, &mut self.service_status) != 0 {
                    if self.service_status.dwCurrentState != SERVICE_STOP_PENDING {
                        break;
                    }
                    thread::sleep(time::Duration::from_millis(250));
                }
            }

            if DeleteService(service.handle) == 0 {
                return Err(Error::new(&format!(
                    "DeleteService: {}",
                    get_last_error_text()
                )));
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

            if ControlService(
                service.handle,
                SERVICE_CONTROL_STOP,
                &mut self.service_status,
            ) != 0
            {
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
    pub fn new(service_name: &str, display_name: &str, description: &str) -> WindowsController {
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
            status_handle: 0,
            controls_accepted: SERVICE_ACCEPT_STOP,
        }
    }

    /// Register the `service_main_wrapper` function, this function is generated by the `Service!` macro.
    ///
    /// Blocks until the service stops and returns the exit code of the service main function.
    /// The exit code is also reported to the SCM.
    pub fn register(
        &mut self,
        service_main_wrapper: WindowsServiceMainWrapperFn,
    ) -> Result<u8, Error> {
        unsafe {
            let mut service_name = get_utf16(self.service_name.as_str());

            // StartServiceCtrlDispatcherW requires the last table entry to have null members.
            // Without it, the dispatcher reads past the table into unrelated stack memory.
            let service_table = [
                SERVICE_TABLE_ENTRYW {
                    lpServiceName: service_name.as_mut_ptr(),
                    lpServiceProc: Some(service_main_wrapper),
                },
                SERVICE_TABLE_ENTRYW {
                    lpServiceName: ptr::null_mut(),
                    lpServiceProc: None,
                },
            ];

            match StartServiceCtrlDispatcherW(service_table.as_ptr()) {
                0 => Err(Error::new("StartServiceCtrlDispatcher")),
                _ => Ok(SERVICE_EXIT_CODE.load(Ordering::Acquire)),
            }
        }
    }
}

/// Status handle of the running service and the checkpoint of its current pending state.
struct ServiceStatusState {
    handle: SERVICE_STATUS_HANDLE,
    check_point: DWORD,
    /// Set once `SERVICE_STOPPED` is reported; later status updates are dropped.
    stopped: bool,
}

/// Context passed to `service_handler`.
struct ServiceContext<T> {
    tx: mpsc::Sender<ServiceEvent<T>>,
    status: Mutex<ServiceStatusState>,
}

impl<T> ServiceContext<T> {
    fn set_service_status(&self, current_state: DWORD, wait_hint: DWORD, service_exit_code: DWORD) {
        let mut status = self.status.lock().unwrap_or_else(PoisonError::into_inner);
        set_service_status(&mut status, current_state, wait_hint, service_exit_code);
    }
}

fn set_service_status(
    status: &mut ServiceStatusState,
    current_state: DWORD,
    wait_hint: DWORD,
    service_exit_code: DWORD,
) {
    // A control (e.g. STOP) may race with the service exiting on its own; never leave STOPPED.
    if status.stopped {
        return;
    }
    status.stopped = current_state == SERVICE_STOPPED;
    // Pending states report an incrementing checkpoint, other states report 0 for both fields.
    let (check_point, wait_hint) = match current_state {
        SERVICE_START_PENDING
        | SERVICE_STOP_PENDING
        | SERVICE_PAUSE_PENDING
        | SERVICE_CONTINUE_PENDING => {
            status.check_point = status.check_point.wrapping_add(1).max(1);
            (status.check_point, wait_hint)
        }
        _ => {
            status.check_point = 0;
            (0, 0)
        }
    };
    let (win32_exit_code, service_specific_exit_code) = if service_exit_code != 0 {
        (ERROR_SERVICE_SPECIFIC_ERROR, service_exit_code)
    } else {
        (0, 0)
    };
    let controls_accepted = match current_state {
        SERVICE_START_PENDING | SERVICE_STOP_PENDING | SERVICE_STOPPED => 0,
        _ => {
            SERVICE_ACCEPT_STOP
                | SERVICE_ACCEPT_SHUTDOWN
                | SERVICE_ACCEPT_PAUSE_CONTINUE
                | SERVICE_ACCEPT_SESSIONCHANGE
        }
    };
    let mut service_status = SERVICE_STATUS {
        dwServiceType: SERVICE_WIN32_OWN_PROCESS,
        dwCurrentState: current_state,
        dwControlsAccepted: controls_accepted,
        dwWin32ExitCode: win32_exit_code,
        dwServiceSpecificExitCode: service_specific_exit_code,
        dwCheckPoint: check_point,
        dwWaitHint: wait_hint,
    };
    unsafe {
        SetServiceStatus(status.handle, &mut service_status);
    }
}

unsafe extern "system" fn service_handler<T>(
    control: DWORD,
    event_type: DWORD,
    event_data: LPVOID,
    context: LPVOID,
) -> DWORD {
    // SAFETY: `context` is the `ServiceContext<T>` allocated by `dispatch`, which is never freed.
    let context = &*(context as *const ServiceContext<T>);
    let tx = &context.tx;

    match control {
        SERVICE_CONTROL_STOP | SERVICE_CONTROL_SHUTDOWN => {
            // Reported once: the service main function has no way to report shutdown progress,
            // and renewing the checkpoint on a timer would hide a hung shutdown from the SCM.
            context.set_service_status(SERVICE_STOP_PENDING, STOP_PENDING_WAIT_HINT_MS, 0);
            let _ = tx.send(ServiceEvent::Stop);
            0
        }
        SERVICE_CONTROL_PAUSE => {
            let _ = tx.send(ServiceEvent::Pause);
            0
        }
        SERVICE_CONTROL_CONTINUE => {
            let _ = tx.send(ServiceEvent::Continue);
            0
        }
        SERVICE_CONTROL_SESSIONCHANGE => {
            let event = event_type;
            let session_notification = event_data as *const WTSSESSION_NOTIFICATION;
            let session_id = (*session_notification).dwSessionId;
            let session = Session::new(session_id);

            if event == WTS_CONSOLE_CONNECT {
                let _ = tx.send(ServiceEvent::SessionConnect(session));
                0
            } else if event == WTS_CONSOLE_DISCONNECT {
                let _ = tx.send(ServiceEvent::SessionDisconnect(session));
                0
            } else if event == WTS_REMOTE_CONNECT {
                let _ = tx.send(ServiceEvent::SessionRemoteConnect(session));
                0
            } else if event == WTS_REMOTE_DISCONNECT {
                let _ = tx.send(ServiceEvent::SessionRemoteDisconnect(session));
                0
            } else if event == WTS_SESSION_LOGON {
                let _ = tx.send(ServiceEvent::SessionLogon(session));
                0
            } else if event == WTS_SESSION_LOGOFF {
                let _ = tx.send(ServiceEvent::SessionLogoff(session));
                0
            } else if event == WTS_SESSION_LOCK {
                let _ = tx.send(ServiceEvent::SessionLock(session));
                0
            } else if event == WTS_SESSION_UNLOCK {
                let _ = tx.send(ServiceEvent::SessionUnlock(session));
                0
            } else {
                0
            }
        }
        _ => ERROR_CALL_NOT_IMPLEMENTED,
    }
}

fn get_args(argc: DWORD, argv: *mut PWSTR) -> Vec<String> {
    let mut args = Vec::new();
    for i in 0..argc {
        unsafe {
            let s = *argv.add(i as usize);
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
        let mut filename = [0u16; MAX_PATH as usize];
        let _size = GetModuleFileNameW(0, filename.as_mut_ptr(), filename.len() as DWORD);
        String::from_utf16(&filename).unwrap_or_else(|_| String::from(""))
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
macro_rules! Service {
    ($name:expr, $function:ident) => {
        use std::ffi::c_void;
        type DWORD = u32;
        type PWSTR = *mut u16;

        extern "system" fn service_main_wrapper(argc: DWORD, argv: *mut PWSTR) {
            dispatch($function, $name, argc, argv);
        }
    };
}

// `T: Send` because the control handler sends events from the SCM dispatcher thread.
#[doc(hidden)]
pub fn dispatch<T: Send>(
    service_main: ServiceMainFn<T>,
    name: &str,
    argc: DWORD,
    argv: *mut PWSTR,
) {
    let args = get_args(argc, argv);
    let service_name = get_utf16(name);
    let (tx, rx) = mpsc::channel();
    let _tx = tx.clone();
    // Leaked on purpose: the handler may be invoked by the dispatcher thread for as long as
    // the service is registered, so the context must outlive this function.
    let context_ptr = Box::into_raw(Box::new(ServiceContext {
        tx,
        status: Mutex::new(ServiceStatusState {
            handle: 0,
            check_point: 0,
            stopped: false,
        }),
    }));
    // SAFETY: `context_ptr` comes from `Box::into_raw` and is never freed.
    let context = unsafe { &*context_ptr };
    {
        // Hold the lock until the handle is stored so the handler never observes an unset handle.
        let mut status = context
            .status
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        status.handle = unsafe {
            RegisterServiceCtrlHandlerExW(
                service_name.as_ptr(),
                Some(service_handler::<T>),
                context_ptr as LPVOID,
            )
        };
        set_service_status(
            &mut status,
            SERVICE_START_PENDING,
            START_PENDING_WAIT_HINT_MS,
            0,
        );
        set_service_status(&mut status, SERVICE_RUNNING, 0, 0);
    }
    let exit_code = service_main(rx, _tx, args, false);
    // Store before reporting SERVICE_STOPPED: StartServiceCtrlDispatcherW may return right after.
    SERVICE_EXIT_CODE.store(exit_code, Ordering::Release);
    context.set_service_status(SERVICE_STOPPED, 0, u32::from(exit_code));
}
