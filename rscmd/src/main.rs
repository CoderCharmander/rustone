extern crate pretty_env_logger;
#[macro_use]
extern crate log;
extern crate rustone;

use clap::{load_yaml, App};

mod actions;
mod cli;

fn enable_ansi() {
    #[cfg(windows)]
    ansi_term::enable_ansi_support();
}

fn main() {
    pretty_env_logger::init();

    let yaml = load_yaml!("cli.yaml");
    let matches = App::from_yaml(yaml).get_matches();

    let out = match matches.subcommand() {
        ("download", matches) => actions::download(matches.unwrap()),
        ("list", _) => actions::list(),
        ("create", matches) => actions::create(matches.unwrap()),
        ("start", matches) => actions::start(matches.unwrap()),
        ("remove", matches) => actions::remove(matches.unwrap()),
        ("upgrade", _) => actions::upgrade(),
        _ => unreachable!(),
    };

    enable_ansi();

    if let Err(err) = out {
        error!("Error! {}", err);

        for e in err.iter().skip(1) {
            error!("caused by: {}", e);
        }

        // Manual backtrace implementation, don't ask please
        if let Some(backtrace) = err.backtrace() {
            error!("backtrace: ");
            for (i, frame) in backtrace.frames().iter().enumerate() {
                error!("{}:", i);
                for symbol in frame.symbols() {
                    error!(
                        " - {} @ {} : {}",
                        symbol.name().map_or_else(
                            || symbol
                                .addr()
                                .map_or_else(|| String::from("?"), |a| format!("{:p}", a)),
                            |n| n.to_string()
                        ),
                        symbol
                            .filename()
                            .map_or_else(|| "?", |n| n.to_str().unwrap_or("?")),
                        symbol
                            .lineno()
                            .map_or_else(|| String::from("?"), |n| n.to_string())
                    );
                }
            }
        }
        std::process::exit(1);
    }
}
