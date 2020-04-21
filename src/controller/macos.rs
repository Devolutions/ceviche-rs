use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc;

use ctrlc;
use log::info;

use crate::controller::{ControllerInterface, ServiceMainFn};
use crate::session;
use crate::Error;
use crate::ServiceEvent;

type MacosServiceMainWrapperFn = extern "system" fn(args: Vec<String>);
pub type Session = session::Session_<u32>;

fn gen_service_plist(name: &str) -> String {
    format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{0}</string>
    <key>ProgramArguments</key>
    <array>
	    <string>/usr/local/bin/{0}</string>
    </array>
</dict>
</plist>
"#, name)
}

fn launchctl_load_daemon(plist_path: &Path) -> Result<(), Error> {
    let output = Command::new("launchctl")
        .arg("load")
        .arg(&plist_path.to_str().unwrap())
        .output()
        .map_err(|e| {
            Error::new(&format!(
                "Failed to load plist {}: {}",
                plist_path.display(),
                e
            ))
        })?;
    if output.stdout.len() > 0 {
        info!("{}", String::from_utf8_lossy(&output.stdout));
    }
    Ok(())
}

fn launchctl_unload_daemon(plist_path: &Path) -> Result<(), Error> {
    let output = Command::new("launchctl")
        .arg("unload")
        .arg(&plist_path.to_str().unwrap())
        .output()
        .map_err(|e| {
            Error::new(&format!(
                "Failed to unload plist {}: {}",
                plist_path.display(),
                e
            ))
        })?;
    if output.stdout.len() > 0 {
        info!("{}", String::from_utf8_lossy(&output.stdout));
    }
    Ok(())
}

fn launchctl_start_daemon(name: &str) -> Result<(), Error> {
    let output = Command::new("launchctl")
        .arg("start")
        .arg(name)
        .output()
        .map_err(|e| Error::new(&format!("Failed to start {}: {}", name, e)))?;
    if output.stdout.len() > 0 {
        info!("{}", String::from_utf8_lossy(&output.stdout));
    }
    Ok(())
}

fn launchctl_stop_daemon(name: &str) -> Result<(), Error> {
    let output = Command::new("launchctl")
        .arg("stop")
        .arg(name)
        .output()
        .map_err(|e| Error::new(&format!("Failed to stop {}: {}", name, e)))?;
    if output.stdout.len() > 0 {
        info!("{}", String::from_utf8_lossy(&output.stdout));
    }
    Ok(())
}

pub struct MacosController {
    /// Manages the service on the system.
    pub service_name: String,
    pub display_name: String,
    pub description: String,
    pub plist: Option<String>,
}

impl MacosController {
    pub fn new(service_name: &str, display_name: &str, description: &str) -> MacosController {
        MacosController {
            service_name: service_name.to_string(),
            display_name: display_name.to_string(),
            description: description.to_string(),
            plist: None,
        }
    }

    /// Register the `service_main_wrapper` function, this function is generated by the `Service!` macro.
    pub fn register(
        &mut self,
        service_main_wrapper: MacosServiceMainWrapperFn,
    ) -> Result<(), Error> {
        service_main_wrapper(env::args().collect());
        Ok(())
    }

    fn write_plist(&self, path: &Path) -> Result<(), Error> {
        info!("Writing plist file {}", path.display());
        let plist_content =
        match &self.plist {
            Some(ref plist) => plist.to_string(),
            None => gen_service_plist(&self.service_name),
        };
        File::create(path)
            .and_then(|mut file| file.write_all(plist_content.as_bytes()))
            .map_err(|e| Error::new(&format!("Failed to write {}: {}", path.display(), e)))

    }
}

impl ControllerInterface for MacosController {
    /// Creates the service on the system.
    fn create(&mut self) -> Result<(), Error> {
        let plist_path =
            Path::new("/Library/LaunchDaemons/").join(format!("{}.plist", &self.service_name));
            
        self.write_plist(&plist_path)?;
        launchctl_load_daemon(&plist_path)
    }
    /// Deletes the service.
    fn delete(&mut self) -> Result<(), Error> {
        let plist_path =
            Path::new("/Library/LaunchDaemons/").join(format!("{}.plist", &self.service_name));
        launchctl_unload_daemon(&plist_path)?;
        fs::remove_file(&plist_path)
            .map_err(|e| Error::new(&format!("Failed to delete {}: {}", plist_path.display(), e)))
    }
    /// Starts the service.
    fn start(&mut self) -> Result<(), Error> {
        launchctl_start_daemon(&self.service_name)
    }
    /// Stops the service.
    fn stop(&mut self) -> Result<(), Error> {
        launchctl_stop_daemon(&self.service_name)
    }
}

/// Generates a `service_main_wrapper` that wraps the provided service main function.
#[macro_export]
macro_rules! Service {
    ($name:expr, $function:ident) => {
        extern "system" fn service_main_wrapper(args: Vec<String>) {
            dispatch($function, args);
        }
    };
}

#[doc(hidden)]
pub fn dispatch<T: Send + 'static>(service_main: ServiceMainFn<T>, args: Vec<String>) {
    let (tx, rx) = mpsc::channel();
    let _tx = tx.clone();

    ctrlc::set_handler(move || {
        let _ = tx.send(ServiceEvent::Stop);
    }).expect("Failed to register Ctrl-C handler");
    service_main(rx, _tx, args, false);
}
