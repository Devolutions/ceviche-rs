use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc;

use ctrlc;
use log::{debug, info, warn};

use crate::controller::{ControllerInterface, ServiceMainFn};
use crate::session;
use crate::Error;
use crate::ServiceEvent;

#[cfg(feature = "systemd-rs")]
use {
    systemd_rs::login::monitor::{Category, Monitor},
    systemd_rs::login::session as login_session,
};

type LinuxServiceMainWrapperFn = fn(args: Vec<String>) -> u8;
pub type Session = session::Session_<String>;

fn systemctl_execute(args: &[&str]) -> Result<(), Error> {
    let mut process = Command::new("systemctl");
    process.args(args);

    let output = process
        .output()
        .map_err(|e| Error::new(&format!("Failed to execute command {}: {}", args[0], e)))?;

    if !output.status.success() {
        return Err(Error::new(&format!(
            "Command \"{}\" failed ({}): {}",
            args[0],
            output.status.code().expect("Process terminated by signal"),
            std::str::from_utf8(&output.stderr).unwrap_or_default()
        )));
    }

    if !output.stdout.is_empty() {
        info!("{}", String::from_utf8_lossy(&output.stdout));
    }

    Ok(())
}

fn systemd_install_daemon(name: &str) -> Result<(), Error> {
    systemctl_execute(&["daemon-reload"])?;
    systemctl_execute(&["enable", name])
}

fn systemd_uninstall_daemon(name: &str) -> Result<(), Error> {
    systemctl_execute(&["disable", name])?;
    systemctl_execute(&["daemon-reload"]).map_err(|e| debug!("{}", e)).ok();
    systemctl_execute(&["reset-failed"]).map_err(|e| debug!("{}", e)).ok();

    Ok(())
}

fn systemd_start_daemon(name: &str) -> Result<(), Error> {
    systemctl_execute(&["start", name])
}

fn systemd_stop_daemon(name: &str) -> Result<(), Error> {
    systemctl_execute(&["stop", name])
}

/// Detect the systemd system unit directory at runtime.
///
/// # Rationale
///
/// This isn't the best approach for Linux — packagers should normally choose the
/// destination and rely on distro tooling (e.g., Debian's dh_installsystemd or
/// RPM's %{_unitdir} macros). Using pkg-config at build/packaging time is a
/// pragmatic, good-enough approach in many situations to discover the vendor
/// unit dir without hardcoding paths.
///
/// # Caveat
///
/// We can't automatically determine whether it should go into user/ or
/// system/, and we default to system/. Use CEVICHE_SYSTEMD_UNITDIR if you need
/// to override this behavior.
///
/// # Detection order
///
/// 1. CEVICHE_SYSTEMD_UNITDIR environment variable (takes precedence)
/// 2. pkg-config --variable=systemdsystemunitdir systemd
/// 3. Fallback probing: /usr/lib/systemd/system, then /lib/systemd/system
fn detect_systemd_unit_dir() -> Result<PathBuf, Error> {
    // 1. Check for environment variable override.
    if let Ok(dir) = env::var("CEVICHE_SYSTEMD_UNITDIR") {
        if !dir.is_empty() {
            info!("Using systemd unit directory from CEVICHE_SYSTEMD_UNITDIR: {dir}");
            return Ok(PathBuf::from(dir));
        }
    }

    // 2. Try pkg-config.
    match Command::new("pkg-config")
        .args(["--variable=systemdsystemunitdir", "systemd"])
        .output()
    {
        Ok(output) if output.status.success() => {
            let dir = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !dir.is_empty() {
                info!("Detected systemd unit directory via pkg-config: {dir}");
                return Ok(PathBuf::from(dir));
            }
        }
        Ok(_) => {
            debug!("pkg-config returned no systemd unit directory");
        }
        Err(e) => {
            debug!("pkg-config not available or failed: {e}");
        }
    }

    // 3. Fallback: probe common directories.
    warn!(
        "pkg-config unavailable or didn't return a systemd unit directory. \
         Falling back to heuristic probing of common vendor directories. \
         This may be distro-specific. Consider setting CEVICHE_SYSTEMD_UNITDIR \
         environment variable to specify the correct path."
    );

    let candidates = ["/usr/lib/systemd/system", "/lib/systemd/system"];

    for &candidate in &candidates {
        let path = Path::new(candidate);
        if path.exists() && path.is_dir() {
            info!("Found systemd unit directory via fallback probing: {candidate}");
            return Ok(PathBuf::from(candidate));
        }
    }

    Err(Error::new(
        "Unable to detect systemd unit directory. \
         Please set CEVICHE_SYSTEMD_UNITDIR environment variable to specify the correct path.",
    ))
}

pub struct LinuxController {
    pub service_name: String,
    pub display_name: String,
    pub description: String,
    pub config: Option<String>,
}

impl LinuxController {
    pub fn new(service_name: &str, display_name: &str, description: &str) -> LinuxController {
        LinuxController {
            service_name: service_name.to_string(),
            display_name: display_name.to_string(),
            description: description.to_string(),
            config: None,
        }
    }

    pub fn register(&mut self, service_main_wrapper: LinuxServiceMainWrapperFn) -> Result<std::process::ExitCode, Error> {
        Ok(std::process::ExitCode::from(service_main_wrapper(env::args().collect())))
    }

    fn get_service_file_name(&self) -> String {
        format!("{}.service", &self.service_name)
    }

    fn get_service_unit_path(&self) -> Result<PathBuf, Error> {
        let unit_dir = detect_systemd_unit_dir()?;
        Ok(unit_dir.join(self.get_service_file_name()))
    }

    fn get_service_dropin_dir(&self) -> Result<PathBuf, Error> {
        let unit_dir = detect_systemd_unit_dir()?;
        Ok(unit_dir.join(format!("{}.d", self.get_service_file_name())))
    }

    fn get_service_unit_content(&self) -> Result<String, Error> {
        Ok(format!(
            r#"
[Unit]
Description={}

[Service]
ExecStart={}

[Install]
WantedBy=multi-user.target"#,
            self.service_name,
            fs::read_link("/proc/self/exe")
                .map_err(|e| Error::new(&format!("Failed to read /proc/self/exe: {}", e)))?
                .to_str()
                .ok_or("Failed to parse /proc/self/exe")?
        ))
    }

    fn write_service_config(&self) -> Result<(), Error> {
        let path = self.get_service_unit_path()?;
        let content = self.get_service_unit_content()?;
        info!("Writing service file {}", path.display());
        File::create(&path)
            .and_then(|mut file| file.write_all(content.as_bytes()))
            .map_err(|e| Error::new(&format!("Failed to write {}: {}", path.display(), e)))?;

        if let Some(ref config) = self.config {
            let dropin_dir = self.get_service_dropin_dir()?;
            let path = dropin_dir.join(format!("{}.conf", self.service_name));

            if !Path::exists(&dropin_dir) {
                fs::create_dir(&dropin_dir)
                    .map_err(|e| Error::new(&format!("Failed to create {}: {}", path.display(), e)))?;
            }
            info!("Writing config file {}", path.display());
            File::create(&path)
                .and_then(|mut file| file.write_all(config.as_bytes()))
                .map_err(|e| Error::new(&format!("Failed to write {}: {}", path.display(), e)))?;
        }

        Ok(())
    }
}

impl ControllerInterface for LinuxController {
    fn create(&mut self) -> Result<(), Error> {
        self.write_service_config()?;

        systemd_install_daemon(&self.service_name)
    }

    fn delete(&mut self) -> Result<(), Error> {
        systemd_uninstall_daemon(&self.service_name)?;

        if let Ok(path) = self.get_service_unit_path() {
            fs::remove_file(&path)
                .map_err(|e| debug!("Failed to delete {}: {}", path.display(), e))
                .ok();
        }

        if let Ok(dropin_dir) = self.get_service_dropin_dir() {
            fs::remove_dir_all(&dropin_dir)
                .map_err(|e| debug!("Failed to delete {}: {}", dropin_dir.display(), e))
                .ok();
        }

        Ok(())
    }

    fn start(&mut self) -> Result<(), Error> {
        systemd_start_daemon(&self.service_name)
    }

    fn stop(&mut self) -> Result<(), Error> {
        systemd_stop_daemon(&self.service_name)
    }
}

#[cfg(feature = "systemd-rs")]
fn run_monitor<T: Send + 'static>(tx: mpsc::Sender<ServiceEvent<T>>) -> Result<Monitor, std::io::Error> {
    let monitor = Monitor::new()?;

    let mut current_session = match login_session::get_active_session() {
        Ok(s) => Some(s),
        Err(e) => {
            debug!("Failed to get active session {}", e);
            None
        }
    };

    monitor.init(Category::Sessions, move || {
        let active_session = match login_session::get_active_session() {
            Ok(s) => Some(s),
            Err(e) => {
                debug!("Failed to get active session {}", e);
                None
            }
        };

        let session_changed = match (&current_session, &active_session) {
            (Some(current_session), Some(active_session)) => current_session != active_session,
            (None, None) => false,
            _ => true,
        };

        if session_changed {
            if let Some(active_session) = active_session.as_ref() {
                let _ = tx.send(ServiceEvent::SessionConnect(Session::new(
                    active_session.identifier.to_string(),
                )));
            }

            if let Some(current_session) = current_session.as_ref() {
                let _ = tx.send(ServiceEvent::SessionDisconnect(Session::new(
                    current_session.identifier.to_string(),
                )));
            }
        }

        current_session = active_session;
    })?;

    Ok(monitor)
}

#[macro_export]
macro_rules! Service {
    ($name:expr, $function:ident) => {
        fn service_main_wrapper(args: Vec<String>) -> u8 {
            dispatch($function, args)
        }
    };
}

#[doc(hidden)]
pub fn dispatch<T: Send + 'static>(
    service_main: ServiceMainFn<T>,
    args: Vec<String>,
) -> u8 {
    let (tx, rx) = mpsc::channel();

    #[cfg(feature = "systemd-rs")]
    {
        let _monitor = run_monitor(tx.clone()).expect("Failed to run session monitor");
    }

    let _tx = tx.clone();

    ctrlc::set_handler(move || {
        let _ = tx.send(ServiceEvent::Stop);
    })
    .expect("Failed to register Ctrl-C handler");
    service_main(rx, _tx, args, false)
}
