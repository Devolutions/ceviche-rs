
#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate log4rs;

#[macro_use]
extern crate ceviche;

use ceviche::controller::*;
use clap::App;
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Root};
use winapi::um::winsvc::{SERVICE_CONTROL_SHUTDOWN, SERVICE_CONTROL_STOP};

static SERVICE_NAME: &'static str = "foobar";
static DISPLAY_NAME: &'static str = "FooBar Service";
static DESCRIPTION: &'static str = "This is the FooBar service";

fn init_logging(standalone_mode: bool) -> Option<()> {
    if standalone_mode {
        let stdout = ConsoleAppender::builder().build();

        let config = Config::builder()
            .appender(Appender::builder().build("stdout", Box::new(stdout)))
            .build(Root::builder().appender("stdout").build(LevelFilter::Info)).ok()?;

        log4rs::init_config(config).ok()?;
    } else {
        let file_appender = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S)} {M} [{h({l})}] - {m}{n}")))
            .build("C:\\Windows\\Temp\\foobar.log").ok()?;

        let config = Config::builder()
            .appender(Appender::builder().build("file_appender", Box::new(file_appender)))
            .build(Root::builder().appender("file_appender").build(LevelFilter::Info)).ok()?;
        
        log4rs::init_config(config).ok()?;
    }

    Some(())
}


fn my_service_main(rx: mpsc::Receiver<DWORD>, args: Vec<String>, standalone_mode: bool) -> u32 {
    init_logging(standalone_mode);
    info!("foobar service started");
    info!("args: {:?}", args);

    loop {
        match rx.recv() {
            Ok(control_code) => {
                if control_code == SERVICE_CONTROL_STOP || control_code == SERVICE_CONTROL_SHUTDOWN {
                    break;
                }
            }
            _ => (),
        }
    }

    info!("foobar service stopping");
    0
}

Service!("Foobar", my_service_main);

fn main() {
    let yaml = load_yaml!("cli.yml");
    let app = App::from_yaml(yaml);
    let matches = app.version(crate_version!()).get_matches();
    let cmd = matches.value_of("cmd").unwrap_or("").to_string();

    let mut controller = Controller::new(SERVICE_NAME, DISPLAY_NAME, DESCRIPTION);

    match cmd.as_str() {
        "create" => {
            let _result = controller.create();
        }
        "delete" => {
            let _result = controller.delete();
        }
        "start" => {
            let _result = controller.start();
        }
        "stop" => {
            let _result = controller.stop();
        }
        "standalone" => {
            let (_tx, rx) = mpsc::channel();
            my_service_main(rx, vec![], true);
        }
        _ => {
            let _result = controller.register(service_main_wrapper);
        }
    }
}
