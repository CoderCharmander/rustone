use getopts::Options;
use servo::{config, paper_api};
use std::env::Args;
use std::fs;

pub fn help(_args: Args) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Usage: servcmd <action> [args...]");
    Ok(())
}

pub fn download(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let mut options = Options::new();
    options.optopt("o", "output", "output file", "OUTPUT");
    let matches = options.parse(&args.collect::<Vec<String>>()[0..])?;

    let version = if !matches.free.is_empty() {
        config::ServerVersion::new(matches.free.iter().next().unwrap_or(&("1.12.2".to_owned())))?
    } else {
        print!(
            "{}",
            options.usage("Usage: servcmd download VERSION [options]")
        );
        return Ok(());
    };

    let output = matches.opt_str("o").unwrap_or(format!(
        "paper-{}.{}.{}.jar",
        version.minecraft.0, version.minecraft.1, version.minecraft.2
    ));

    let mut file = fs::File::create(output)?;
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(paper_api::ProjectVersionList::download(version, &mut file))?;

    Ok(())
}
