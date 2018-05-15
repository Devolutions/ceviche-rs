extern crate gcc;

fn main() {
    gcc::Build::new().file("src/main.c").include("src").compile("libmain.a");
}
