#[macro_use]
extern crate log;
extern crate which;

extern crate serde;
extern crate serde_json;
use serde::{Serialize, Deserialize};

use std::env;
use std::sync::mpsc;
use std::path::{PathBuf};
use std::process::{Command};
use std::fs::File;
use std::io::BufReader;

use ceviche::controller::*;
use ceviche::{Service, ServiceEvent};

use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;

mod pwsh;
use pwsh::*;

#[derive(Serialize, Deserialize)]
pub struct CmdletService {
    #[serde(rename = "ServiceName")]
    pub service_name: String,
    #[serde(rename = "DisplayName")]
    pub display_name: String,
    #[serde(rename = "Description")]
    pub description: String,
    #[serde(rename = "CompanyName")]
    pub company_name: String,
    #[serde(rename = "WorkingDir")]
    pub working_dir: String,
    #[serde(rename = "ModuleName")]
    pub module_name: String,
    #[serde(rename = "StartCommand")]
    pub start_command: String,
    #[serde(rename = "StopCommand")]
    pub stop_command: String,
}

#[derive(Serialize, Deserialize)]
pub struct ServiceManifest {
    #[serde(rename = "ServiceName")]
    pub service_name: String,
    #[serde(rename = "DisplayName")]
    pub display_name: Option<String>,
    #[serde(rename = "Description")]
    pub description: Option<String>,
    #[serde(rename = "CompanyName")]
    pub company_name: Option<String>,
    #[serde(rename = "WorkingDir")]
    pub working_dir: String,
    #[serde(rename = "ModuleName")]
    pub module_name: Option<String>,
    #[serde(rename = "StartCommand")]
    pub start_command: String,
    #[serde(rename = "StopCommand")]
    pub stop_command: String,
}

impl ServiceManifest {
    pub fn get_module_name(&self) -> &str {
        if let Some(module_name) = &self.module_name {
            return module_name.as_str();
        }
        return self.service_name.as_str();
    }
}

enum CustomServiceEvent {}

pub fn get_base_name() -> Option<String> {
    let current_exe = std::env::current_exe().ok()?;
    let base_name = current_exe.as_path().file_stem()?.to_str()?;
    return Some(base_name.to_string());
}

pub fn get_service_manifest() -> Option<ServiceManifest> {
    let base_name = get_base_name()?;
    let mut manifest_path = std::env::current_exe().ok()?;
    let manifest_name = format!("{}.service.json", base_name);
    manifest_path.set_file_name(manifest_name.as_str());
    if !manifest_path.exists() {
        let manifest_name = "service.json".to_string();
        manifest_path.set_file_name(manifest_name.as_str());
    }
    let file = File::open(manifest_path.as_path()).ok()?;
    let result = serde_json::from_reader(BufReader::new(file));
    result.ok()
}

impl CmdletService {
    fn load() -> Option<Self> {
        let service_manifest = get_service_manifest()?;
        let module_name = service_manifest.get_module_name().to_string();
        let module_manifest = get_module_manifest(&module_name)?;
    
        let service_name = service_manifest.service_name.to_string();
        let display_name = service_manifest.display_name.unwrap_or(service_name.to_string());
        let description = service_manifest.description.unwrap_or(module_manifest.description.to_string());
        let company_name = service_manifest.company_name.unwrap_or(module_manifest.company_name.to_string());
        let working_dir = service_manifest.working_dir.to_string();
        let start_command = service_manifest.start_command.to_string();
        let stop_command = service_manifest.stop_command.to_string();
    
        Some(CmdletService {
            service_name: service_name.to_string(),
            display_name: display_name.to_string(),
            description: description.to_string(),
            company_name: company_name.to_string(),
            working_dir: working_dir.to_string(),
            module_name: module_name.to_string(),
            start_command: start_command.to_string(),
            stop_command: stop_command.to_string(),
        })
    }

    pub fn get_working_dir(&self) -> Option<PathBuf> {
        let working_dir = expand_str::expand_string_with_env(self.working_dir.as_str()).ok()?;
        return Some(PathBuf::from(working_dir));
    }

    pub fn get_service_name(&self) -> &str {
        self.service_name.as_str()
    }

    pub fn get_description(&self) -> &str {
        self.service_name.as_str()
    }

    pub fn get_module_name(&self) -> &str {
        self.module_name.as_str()
    }

    pub fn get_start_command(&self) -> &str {
        self.start_command.as_str()
    }

    pub fn get_stop_command(&self) -> &str {
        self.stop_command.as_str()
    }

    pub fn start(&self) {
        let cmdlet_name = self.get_module_name();
        let function = self.get_start_command();
        let output = run_cmdlet_function(self, cmdlet_name, &function).unwrap();
        let stdout = String::from_utf8(output.stdout).unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        info!("{}:\n {} {}", function, stdout, stderr);
    }

    pub fn stop(&self) {
        let cmdlet_name = self.get_module_name();
        let function = self.get_stop_command();
        let output = run_cmdlet_function(self, cmdlet_name, &function).unwrap();
        let stdout = String::from_utf8(output.stdout).unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        info!("{}:\n {} {}", function, stdout, stderr);
    }
}

fn init_logging(service: &CmdletService, standalone_mode: bool) -> Option<()> {
    let mut log_path = service.get_working_dir()?;
    let cmdlet_name = service.get_module_name();
    log_path.push(format!("{}.log", cmdlet_name));

    if standalone_mode {
        let stdout = ConsoleAppender::builder().build();

        let config = Config::builder()
            .appender(Appender::builder().build("stdout", Box::new(stdout)))
            .build(Root::builder().appender("stdout").build(LevelFilter::Info))
            .ok()?;

        log4rs::init_config(config).ok()?;
    } else {
        let file_appender = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new(
                "{d(%Y-%m-%d %H:%M:%S)} {M} [{h({l})}] - {m}{n}",
            )))
            .build(log_path)
            .ok()?;

        let config = Config::builder()
            .appender(Appender::builder().build("file_appender", Box::new(file_appender)))
            .build(
                Root::builder()
                    .appender("file_appender")
                    .build(LevelFilter::Info),
            )
            .ok()?;

        log4rs::init_config(config).ok()?;
    }
    Some(())
}

fn run_cmdlet_function(service: &CmdletService, cmdlet: &str, function: &str) -> std::io::Result<std::process::Output> {
    let powershell = find_powershell().unwrap();
    let working_dir = service.get_working_dir().unwrap();

    let command = format!(
        "Import-Module -Name {};\n\
        {}", cmdlet, function);

    Command::new(&powershell)
        .arg("-Command").arg(&command)
        .current_dir(working_dir)
        .output()
}

fn cmdlet_service_main(
    rx: mpsc::Receiver<ServiceEvent<CustomServiceEvent>>,
    _tx: mpsc::Sender<ServiceEvent<CustomServiceEvent>>,
    args: Vec<String>,
    standalone_mode: bool,
) -> u32 {
    let service = CmdletService::load().expect("unable to load service manifest");
    let module_name = service.get_module_name();
    if let Some(module_base) = find_cmdlet_base(module_name) {
        println!("Using module {} from {}", module_name, module_base.as_path().to_str().unwrap());
    }
    init_logging(&service, standalone_mode);
    info!("{} service started", service.get_service_name());
    info!("args: {:?}", args);

    service.start();

    loop {
        if let Ok(control_code) = rx.recv() {
            info!("Received control code: {}", control_code);
            match control_code {
                ServiceEvent::Stop => {
                    service.stop();
                    break
                }
                _ => (),
            }
        }
    }

    info!("{} service stopping", service.get_service_name());
    0
}

Service!("cmdlet", cmdlet_service_main);

fn main() {
    let service = CmdletService::load().unwrap();
    let mut controller = Controller::new(service.service_name.as_str(),
        service.display_name.as_str(), service.description.as_str());

    if let Some(cmd) = env::args().nth(1) {
        match cmd.as_str() {
            "create" => {
                if let Err(e) = controller.create() {
                    println!("{}", e);
                }
            }
            "delete" => {
                if let Err(e) = controller.delete() {
                    println!("{}", e);
                }
            }
            "start" => {
                if let Err(e) = controller.start() {
                    println!("{}", e);
                }
            }
            "stop" => {
                if let Err(e) = controller.stop() {
                    println!("{}", e);
                }
            }
            "run" => {
                let (tx, rx) = mpsc::channel();
                let _tx = tx.clone();
        
                ctrlc::set_handler(move || {
                    let _ = tx.send(ServiceEvent::Stop);
                }).expect("Failed to register Ctrl-C handler");
        
                cmdlet_service_main(rx, _tx, vec![], true);
            }
            _ => {
                println!("invalid command: {}", cmd);
            }
        }
    } else {
        let _result = controller.register(service_main_wrapper);
    }
}
