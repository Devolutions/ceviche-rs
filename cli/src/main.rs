#[macro_use]
extern crate log;
extern crate log4rs;

#[macro_use]
extern crate clap;

extern crate ceviche;

use clap::App;

use ceviche::*;
use ceviche::controller::*;

static SERVICE_NAME: &'static str = "foobar";
static DISPLAY_NAME: &'static str = "FooBar Service";
static DESCRIPTION: &'static str = "This is the FooBar service";

fn main() {
    let yaml = load_yaml!("cli.yml");
    let app = App::from_yaml(yaml);
    let matches = app.version(crate_version!()).get_matches();
    let cmd = matches.value_of("cmd").unwrap_or("").to_string();

	let mut controller = Controller::new(SERVICE_NAME, DISPLAY_NAME, DESCRIPTION).unwrap();

    match cmd.as_str() {
        "create" => {
			controller.create();
        },
        "delete" => {
			controller.delete();
        },
        "start" => {
			controller.start();
        },
        "stop" => {
			controller.stop();
        },
        _ => {
        	controller.register();
        }
    }
}
