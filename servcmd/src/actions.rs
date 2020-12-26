use getopts::Options;
use servo::{
    config,
    error::{Result, ServoError},
    paper_api,
    servers::{CachedJar, Server},
};
use std::env::Args;
use std::fs;

pub fn help(_args: Args) -> Result<()> {
    eprintln!("Usage: servcmd <action> [args...]");
    Ok(())
}

pub fn download(args: Args) -> Result<()> {
    let mut options = Options::new();
    options.optopt("o", "output", "output file", "OUTPUT");
    let matches = options.parse(&args.collect::<Vec<String>>()[0..])?;

    let version = if !matches.free.is_empty() {
        config::ServerVersion::new(matches.free.iter().next().unwrap())?
    } else {
        config::ServerVersion::new("1.12.2").unwrap()
    };

    let output = matches.opt_str("o").unwrap_or(format!(
        "paper-{}.{}.{}.jar",
        version.minecraft.0, version.minecraft.1, version.minecraft.2
    ));

    println!("Downloading version {} into {}", version, output);

    let mut file = fs::File::create(output)?;
    paper_api::ProjectVersionList::download(&version, &mut file)?;

    Ok(())
}

pub fn create(mut args: Args) -> Result<()> {
    let name = args
        .next()
        .ok_or(ServoError::boxnew("Not enough arguments"))?;
    let version = args
        .next()
        .ok_or(ServoError::boxnew("Not enough arguments"))?;
    Server::create(&name, config::ServerVersion::new(&version)?)?;
    Ok(())
}

pub fn start(mut args: Args) -> Result<()> {
    let name = args
        .next()
        .ok_or(ServoError::boxnew("Not enough arguments"))?;
    let server = Server::get(&name)?;
    let jar = CachedJar::download(server.config.version)?;
    let mut child = jar.start_server(server)?;
    child.wait()?;
    Ok(())
}
