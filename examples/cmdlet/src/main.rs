#[macro_use]
extern crate log;
extern crate which;

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

static SERVICE_NAME: &'static str = "CmdletService";
static DISPLAY_NAME: &'static str = "Cmdlet Service";
static DESCRIPTION: &'static str = "PowerShell cmdlet service wrapper";

static CMDLET_VENDOR: &'static str = "PowerShell";
static CMDLET_NAME: &'static str = "CmdletService";

enum CustomServiceEvent {}

pub fn get_config_path() -> PathBuf {
    let program_data = env::var("ProgramData").unwrap();
    let mut config_path = PathBuf::from(program_data);
    config_path.push(CMDLET_VENDOR);
    config_path.push(CMDLET_NAME);
    config_path
}

fn init_logging(standalone_mode: bool) -> Option<()> {
    let mut log_path = get_config_path();
    log_path.push(format!("{}.log", CMDLET_NAME));

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

fn run_cmdlet_function(cmdlet: &str, function: &str) -> std::io::Result<std::process::Output> {
    let powershell = find_powershell().unwrap();
    let config_path = get_config_path();

    let command = format!(
        "Import-Module -Name {};\n\
        {}", cmdlet, function);

    Command::new(&powershell)
        .arg("-Command").arg(&command)
        .current_dir(config_path)
        .output()
}

fn start_cmdlet_service() {
    let function = format!("Start-{}", CMDLET_NAME);
    let output = run_cmdlet_function(CMDLET_NAME, &function).unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    info!("{}: {} / {}", function, stdout, stderr);
}

fn stop_cmdlet_service() {
    let function = format!("Stop-{}", CMDLET_NAME);
    let output = run_cmdlet_function(CMDLET_NAME, &function).unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    info!("{}: {} / {}", function, stdout, stderr);
}

fn cmdlet_service_main(
    rx: mpsc::Receiver<ServiceEvent<CustomServiceEvent>>,
    _tx: mpsc::Sender<ServiceEvent<CustomServiceEvent>>,
    args: Vec<String>,
    standalone_mode: bool,
) -> u32 {
    init_logging(standalone_mode);
    info!("{} service started", CMDLET_NAME);
    info!("args: {:?}", args);

    start_cmdlet_service();

    loop {
        if let Ok(control_code) = rx.recv() {
            info!("Received control code: {}", control_code);
            match control_code {
                ServiceEvent::Stop => {
                    stop_cmdlet_service();
                    break
                }
                _ => (),
            }
        }
    }

    info!("{} service stopping", CMDLET_NAME);
    0
}

Service!("cmdlet", cmdlet_service_main);

fn main() {
    let mut controller = Controller::new(SERVICE_NAME, DISPLAY_NAME, DESCRIPTION);

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
