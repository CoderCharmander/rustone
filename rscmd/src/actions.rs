use crate::cli;
use clap::ArgMatches;
use rustone::{
    cacher::{read_cache_meta, CachedJar},
    config,
    config::ServerVersion,
    errors::*,
    paper_api,
    servers::{get_servers, Server},
};
use std::{borrow::Borrow, fs, io::BufRead, process::Stdio};

pub fn download(args: &ArgMatches) -> Result<()> {
    let mut version = ServerVersion::new(args.value_of("VERSION").unwrap())?;

    let output = format!("paper-{}.jar", version.minecraft);
    let output = args.value_of("output").unwrap_or(&output);

    println!("Downloading version {} into {}", version, output);

    let mut file = fs::File::create(output).chain_err(|| "could not create jar file")?;
    paper_api::ProjectVersionList::download(&mut version, &mut file)?;

    Ok(())
}

pub fn create(args: &ArgMatches) -> Result<()> {
    let name = args.value_of("NAME").unwrap();
    let version = args.value_of("VERSION").unwrap();
    Server::create(&name, config::ServerVersion::new(&version)?)?;
    Ok(())
}

pub fn start(args: &ArgMatches) -> Result<()> {
    let name = args.value_of("NAME").unwrap();
    let server = Server::get(&name)?;
    let jar = CachedJar::get(server.config.version)?;
    let mut child =
        jar.start_server(server, Stdio::inherit(), Stdio::inherit(), Stdio::inherit())?;
    child.wait().chain_err(|| "wait failed")?;
    Ok(())
}

pub fn list() -> Result<()> {
    for server in get_servers()? {
        println!(
            "{} ({})",
            server.config.name,
            cli::SECONDARY.paint(format!("{}", server.config.version))
        );
    }

    Ok(())
}

pub fn remove(args: &ArgMatches) -> Result<()> {
    let server = Server::get(&args.value_of("NAME").unwrap())?;
    let confirm_str = format!(
        "Yes, erase {} completely and irrecoverably.",
        server.config.name
    );
    println!("{} THIS WILL IRRECOVERABLY ERASE {}, ALL OF ITS CONFIGURATION AND WORLDS! TO CONTINUE, TYPE '{}'.",
        cli::WARNING_HEADER_STYLE.paint("WARNING!"),
        cli::SECONDARY.paint(&server.config.name),
        cli::SECONDARY.paint(&confirm_str)
    );
    let input = std::io::stdin().lock().lines().next().unwrap().unwrap();
    if confirm_str == input {
        std::fs::remove_dir_all(server.server_path()?)
            .chain_err(|| "failed to remove server directory")?;
        std::fs::remove_file(server.config_path()?)
            .chain_err(|| "failed to remove server config")?;
    } else {
        println!("Abort.");
    }
    Ok(())
}

pub fn upgrade() -> Result<()> {
    println!("Upgrading jars...");
    for cj in read_cache_meta()?.jars {
        println!("Conditionally upgrading version {}...", cj.mcversion);
        CachedJar::get(ServerVersion {
            minecraft: cj.mcversion,
            patch: None,
        })?;
    }
    Ok(())
}
