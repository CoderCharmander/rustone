extern crate rustone;

use clap::{load_yaml, App};

mod actions;
mod cli;

fn enable_ansi() {
    #[cfg(windows)]
    ansi_term::enable_ansi_support();
}

#[tokio::main]
async fn main() {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from_yaml(yaml).get_matches();

    let out = match matches.subcommand() {
        ("download", matches) => actions::download(matches.unwrap()).await,
        ("list", _) => actions::list(),
        ("create", matches) => actions::create(matches.unwrap()),
        ("start", matches) => actions::start(matches.unwrap()).await,
        ("remove", matches) => actions::remove(matches.unwrap()),
        ("cache", matches) => actions::cache(matches.unwrap()).await,
        _ => unreachable!(),
    };

    enable_ansi();

    if let Err(err) = out {
        println!("{} {}", cli::ERROR_HEADER_STYLE.paint("error!"), err);

        for e in err.iter().skip(1) {
            println!(" {} {}", cli::HIGHLIGHT.paint("caused by:"), e);
        }

        // Manual backtrace implementation, don't ask please
        if let Some(backtrace) = err.backtrace() {
            println!("backtrace: ");
            for (i, frame) in backtrace.frames().iter().enumerate() {
                println!(" {}:", i);
                for symbol in frame.symbols() {
                    println!(
                        "  - {} @ {} : {}",
                        cli::HIGHLIGHT.paint(symbol.name().map_or_else(
                            || {
                                symbol
                                    .addr()
                                    .map_or_else(|| String::from("?"), |a| format!("{:p}", a))
                            },
                            |n| n.to_string()
                        )),
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
