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

use ceviche::controller::*;
use ceviche::{Service, ServiceEvent};

use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;

#[derive(Serialize, Deserialize)]
pub struct ServiceDefinition {
    #[serde(rename = "ServiceName")]
    pub service_name: String,
    #[serde(rename = "DisplayName")]
    pub display_name: String,
    #[serde(rename = "Description")]
    pub description: String,
    #[serde(rename = "CompanyName")]
    pub company_name: String,
}

enum CustomServiceEvent {}

pub fn get_cmdlet_name() -> Option<String> {
    // Detect cmdlet name from service executable name
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(file_stem) = current_exe.as_path().file_stem() {
            if let Some(cmdlet_name) = file_stem.to_str() {
                return Some(cmdlet_name.to_string());
            }
        }
    }
    return None;
}

pub fn get_service_definition() -> ServiceDefinition {
    let service_json = include_str!("service.json");
    let result = serde_json::from_str(&service_json);
    result.unwrap()
}

pub fn get_config_path(service_definition: &ServiceDefinition) -> PathBuf {
    let program_data = env::var("ProgramData").unwrap();
    let mut config_path = PathBuf::from(program_data);
    config_path.push(service_definition.company_name.as_str());
    config_path.push(service_definition.service_name.as_str());
    config_path
}

fn init_logging(service_definition: &ServiceDefinition, standalone_mode: bool) -> Option<()> {
    let mut log_path = get_config_path(service_definition);
    let cmdlet_name = service_definition.service_name.as_str();
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

pub fn find_powershell() -> Option<PathBuf> {
    if let Ok(powershell) = which::which("pwsh") {
        return Some(powershell);
    }
    which::which("powershell").ok()
}

fn run_cmdlet_function(service_definition: &ServiceDefinition, cmdlet: &str, function: &str) -> std::io::Result<std::process::Output> {
    let powershell = find_powershell().unwrap();
    let config_path = get_config_path(service_definition);

    let command = format!(
        "Import-Module -Name {};\n\
        {}", cmdlet, function);

    Command::new(&powershell)
        .arg("-Command").arg(&command)
        .current_dir(config_path)
        .output()
}

fn start_cmdlet_service(service_definition: &ServiceDefinition) {
    let cmdlet_name = service_definition.service_name.as_str();
    let function = format!("Start-{}", cmdlet_name);
    let output = run_cmdlet_function(service_definition, cmdlet_name, &function).unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    info!("{}:\n {} {}", function, stdout, stderr);
}

fn stop_cmdlet_service(service_definition: &ServiceDefinition) {
    let cmdlet_name = service_definition.service_name.as_str();
    let function = format!("Stop-{}", cmdlet_name);
    let output = run_cmdlet_function(service_definition, cmdlet_name, &function).unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    info!("{}:\n {} {}", function, stdout, stderr);
}

fn cmdlet_service_main(
    rx: mpsc::Receiver<ServiceEvent<CustomServiceEvent>>,
    _tx: mpsc::Sender<ServiceEvent<CustomServiceEvent>>,
    args: Vec<String>,
    standalone_mode: bool,
) -> u32 {
    let service_definition = get_service_definition();
    init_logging(&service_definition, standalone_mode);
    info!("{} service started", service_definition.service_name.as_str());
    info!("args: {:?}", args);

    start_cmdlet_service(&service_definition);

    loop {
        if let Ok(control_code) = rx.recv() {
            info!("Received control code: {}", control_code);
            match control_code {
                ServiceEvent::Stop => {
                    stop_cmdlet_service(&service_definition);
                    break
                }
                _ => (),
            }
        }
    }

    info!("{} service stopping", service_definition.service_name.as_str());
    0
}

Service!("cmdlet", cmdlet_service_main);

fn main() {
    let service_definition = get_service_definition();
    let mut controller = Controller::new(service_definition.service_name.as_str(),
        service_definition.display_name.as_str(), service_definition.description.as_str());

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
