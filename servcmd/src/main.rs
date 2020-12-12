extern crate pretty_env_logger;
#[macro_use]
extern crate log;
extern crate servo;

use std::{env, process::exit};

mod actions;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let mut args = env::args();
    let program = args.next().unwrap_or("servcmd".to_owned());
    let action = args.next();
    if let None = action {
        error!("No action specified");
        eprintln!("Usage: {} <action> [args...]", program);
        exit(1);
    }

    match action.unwrap().as_str() {
        "help" => actions::help(args),
        "download" => actions::download(args),
        &_ => actions::help(args),
    }
}
