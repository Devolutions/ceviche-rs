
#[macro_use]
extern crate clap;

extern crate ceviche;

use clap::App;

use ceviche::*;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let app = App::from_yaml(yaml);
    let matches = app.version(crate_version!()).get_matches();
    let cmd = matches.value_of("cmd").unwrap_or("").to_string();

	let mut service = Service::new("foobar", "FooBar Service", "This is the FooBar service").unwrap();

    match cmd.as_str() {
        "create" => {
			service.create();
        },
        "delete" => {
			service.delete();
        },
        "start" => {
			service.start();
        },
        "stop" => {
			service.stop();
        },
        _ => {
        	service.register();
        }
    }
}
