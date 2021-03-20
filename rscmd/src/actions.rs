use crate::{candy, cli};
use clap::ArgMatches;
use rustone::{
    cacher::{self, CachedJarMetaKey},
    config,
    config::ServerVersion,
    errors::*,
    server_kinds::{paper, ServerKind},
    servers::{get_servers, Server},
};
use std::{fs::File, io::BufRead, process::Stdio};

mod cache;

pub async fn download(args: &ArgMatches<'_>) -> Result<()> {
    let mut version = ServerVersion::new(args.value_of("VERSION").unwrap())?;

    let output = format!("paper-{}.jar", version.minecraft);
    let output = args.value_of("output").unwrap_or(&output);

    println!("Downloading version {} into {}", version, output);

    let mut response = paper::download_response(&mut version).await?;
    let mut file = File::create(output).chain_err(|| "failed to create jar file")?;
    candy::download(&mut response, &mut file).await?;

    Ok(())
}

pub fn create(args: &ArgMatches<'_>) -> Result<()> {
    let name = args.value_of("NAME").unwrap();
    let version = args.value_of("VERSION").unwrap();
    let kind = args.value_of("type").unwrap();
    Server::create(
        &name,
        config::ServerVersion::new(&version)?,
        kind.to_owned(),
    )?;
    Ok(())
}

pub async fn start(args: &ArgMatches<'_>) -> Result<()> {
    let name = args.value_of("NAME").unwrap();
    let server = Server::get(&name)?;
    let key = CachedJarMetaKey {
        kind: server.config.kind.clone(),
        version: server.config.version.minecraft,
    };
    let cached_patch = cacher::get_cached_patch(&key)?;
    let kind = server.config.kind.parse::<ServerKind>()?;
    if let Some(cached_patch) = cached_patch {
        println!("Checking for updates...");
        let latest_patch = kind
            .get_latest_patch(&server.config.version.minecraft)
            .await?;
        if latest_patch > cached_patch {
            println!("Found a new build! Downloading...");
            let mut response = kind
                .download_response(&mut server.config.version.clone())
                .await?;
            let mut file =
                File::create(key.path()).chain_err(|| "failed to create file for downloading")?;
            candy::download(&mut response, &mut file).await?;
        } else {
            println!("Server up to date");
        }
    } else {
        println!("Downloading server jar...");
        let latest_patch = kind
            .get_latest_patch(&server.config.version.minecraft)
            .await?;
        let (mut file, _) = cacher::cache_jar(
            server.config.version.minecraft,
            latest_patch,
            server.config.kind.clone(),
        )?;
        let mut version = ServerVersion {
            patch: Some(latest_patch),
            ..server.config.version
        };
        let mut resp = kind.download_response(&mut version).await?;
        candy::download(&mut resp, &mut file).await?;
    }
    let mut child = kind.launch(
        server.config,
        Stdio::inherit(),
        Stdio::inherit(),
        Stdio::inherit(),
    )?;
    println!("Launching...");
    child.wait().await.chain_err(|| "wait failed")?;
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

pub async fn cache(args: &ArgMatches<'_>) -> Result<()> {
    match args.subcommand() {
        ("upgrade", _) => cache::upgrade().await,
        ("purge", _) => cache::purge(),
        _ => unreachable!(),
    }
}
