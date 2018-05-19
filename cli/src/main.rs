#[macro_use]
extern crate log;
extern crate log4rs;

#[macro_use]
extern crate clap;

extern crate ceviche;

use clap::App;

use ceviche::*;
use ceviche::factory::*;
use ceviche::controller::*;
use ceviche::service::*;

static SERVICE_NAME: &'static str = "foobar";
static DISPLAY_NAME: &'static str = "FooBar Service";
static DESCRIPTION: &'static str = "This is the FooBar service";

pub struct MyService {
    pub service_name: String,
}

impl Service for MyService {
    fn main(&self) {
        println!("MyService main!");
    }
}

fn main() {
    let yaml = load_yaml!("cli.yml");
    let app = App::from_yaml(yaml);
    let matches = app.version(crate_version!()).get_matches();
    let cmd = matches.value_of("cmd").unwrap_or("").to_string();

    let mut factory = Factory::new();

    factory.register(SERVICE_NAME, move |args| {
        Box::new(MyService { service_name: SERVICE_NAME.to_string() })
    });

    /*
    if let Some(my_service) = factory.call(SERVICE_NAME) {
        println!("we've got a service!");
        my_service.main();
    } else {
        println!("no soup for you");
    }
    */

	let mut controller = Controller::new(factory, SERVICE_NAME, DISPLAY_NAME, DESCRIPTION).unwrap();

    match cmd.as_str() {
        "create" => {
			controller.create().unwrap();
        },
        "delete" => {
			controller.delete().unwrap();
        },
        "start" => {
			controller.start().unwrap();
        },
        "stop" => {
			controller.stop().unwrap();
        },
        _ => {
        	controller.register().unwrap();
        }
    }
}
