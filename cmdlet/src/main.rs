#[macro_use]
extern crate log;
extern crate which;

extern crate serde;
extern crate serde_json;

extern crate base64;

use std::env;
use std::sync::mpsc;

use ceviche::controller::*;
use ceviche::{Service, ServiceEvent};

use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;

mod pwsh;
mod service;

use service::CmdletService;

enum CustomServiceEvent {}

fn init_logging(service: &CmdletService, standalone_mode: bool) -> Option<()> {
    let mut log_path = service.get_working_dir()?;
    let log_file = service.get_log_file();
    log_path.push(log_file);

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

fn cmdlet_service_main(
    rx: mpsc::Receiver<ServiceEvent<CustomServiceEvent>>,
    _tx: mpsc::Sender<ServiceEvent<CustomServiceEvent>>,
    args: Vec<String>,
    standalone_mode: bool,
) -> u32 {
    let service = CmdletService::load().expect("unable to load service manifest");
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
    let service = CmdletService::load().expect("unable to load cmdlet service");
    let mut controller = Controller::new(service.get_service_name(),
        service.get_display_name(), service.get_description());

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
