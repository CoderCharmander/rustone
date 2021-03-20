use std::{fs, io::Write, process::Stdio};

use error_chain::bail;
use reqwest::StatusCode;
use serde::Deserialize;

use crate::{
    cacher::CachedJarMetaKey,
    config::{MinecraftVersion, ServerVersion},
    errors::*,
    servers::Server,
};

pub struct PaperUpdater;

pub async fn is_latest_patch(version: &MinecraftVersion, patch: u32) -> Result<bool> {
    Ok(get_latest_patch(version).await? <= patch)
}

pub async fn get_latest() -> Result<ServerVersion> {
    let response = reqwest::get("https://papermc.io/api/v1/paper")
        .await
        .chain_err(|| "paper: failed to request version list")?;
    let versions = response
        .json::<ProjectResponse>()
        .await
        .chain_err(|| "paper: failed to decode project info json")?
        .versions;
    let latest = versions
        .iter()
        .max()
        .ok_or_else::<Error, _>(|| "paper: no maximum version found".into())?;
    Ok(ServerVersion {
        minecraft: *latest,
        patch: Some(get_latest_patch(latest).await?),
    })
}

pub async fn download_response(version: &mut ServerVersion) -> Result<reqwest::Response> {
    let (url, p) = get_download_url(version).await?;
    version.patch = Some(p);
    match reqwest::get(url).await {
        Ok(resp) => {
            if resp.status() != StatusCode::NOT_FOUND {
                Ok(resp)
            } else {
                bail!("paper: nonexistant server version {}", version);
            }
        }
        Err(e) => Err(e).chain_err(|| format!("paper: failed to download version {}", version)),
    }
}

pub async fn get_latest_patch(version: &MinecraftVersion) -> Result<u32> {
    let url = format!("https://papermc.io/api/v1/paper/{}", version);
    let response = reqwest::get(url)
        .await
        .chain_err(|| "paper: failed to request build list")?;
    if response.status() == StatusCode::NOT_FOUND {
        bail!("paper: nonexistant minecraft version {}", version);
    }
    let project: ProjectVersionResponse = response
        .json()
        .await
        .chain_err(|| "paper: failed to decode project version info json")?;
    Ok(project.builds.latest)
}

#[derive(Deserialize, Debug)]
struct ProjectResponse {
    pub project: String,
    pub versions: Vec<MinecraftVersion>,
}

#[derive(Deserialize)]
struct PatchList {
    pub latest: u32,
    //all: Vec<u32>,
}

#[derive(Deserialize)]
struct ProjectVersionResponse {
    builds: PatchList,
}

async fn get_download_url(version: &ServerVersion) -> Result<(String, u32)> {
    let patch = match version.patch {
        Some(p) => Ok(p),
        None => get_latest_patch(&version.minecraft).await,
    }?;
    Ok((
        format!(
            "https://papermc.io/api/v1/paper/{}/{}/download",
            version.minecraft, patch
        ),
        patch,
    ))
}

pub struct PaperServer;

pub fn initialize(config: &crate::config::ServerConfig) -> Result<()> {
    for dir in &["configs", "worlds", "plugins"] {
        let path = config.path().join(dir);
        fs::create_dir_all(&path).chain_err(|| {
            format!(
                "paper: failed to create '{}' directory @ {}",
                dir,
                path.to_string_lossy()
            )
        })?;
    }
    let mut eula_file = fs::File::create(config.path().join("configs").join("eula.txt"))
        .chain_err(|| "paper: failed to create eula file")?;
    writeln!(&mut eula_file, "eula=true").chain_err(|| "paper: failed to write into eula file")?;
    Ok(())
}

pub fn launch(
    config: crate::config::ServerConfig,
    stdout: Stdio,
    stdin: Stdio,
    stderr: Stdio,
) -> Result<tokio::process::Child> {
    let child = tokio::process::Command::new("java")
        .args(&config.extra_java_args)
        .arg("-jar")
        .arg(
            CachedJarMetaKey {
                kind: "paper".into(),
                version: config.version.minecraft,
            }
            .path(),
        )
        .args(server_args(&config)?)
        .args(&config.extra_server_args)
        .current_dir(Server { config }.server_path()?.join("configs"))
        .stdout(stdout)
        .stdin(stdin)
        .stderr(stderr)
        .spawn()
        .chain_err(|| "failed to spawn server process")?;
    Ok(child)
}

fn server_args(server: &crate::config::ServerConfig) -> Result<Vec<String>> {
    let config_path = server
        .path()
        .join("configs")
        .canonicalize()
        .chain_err(|| "canonicalize failed")?;
    let world_path = server
        .path()
        .join("worlds")
        .canonicalize()
        .chain_err(|| "canonicalize failed")?;
    let plugins_path = server
        .path()
        .join("plugins")
        .canonicalize()
        .chain_err(|| "canonicalize failed")?;

    Ok(vec![
        // Don't open a GUI, that could interfere with us
        "--nogui".to_string(),
        // Config files
        "--paper-settings".to_string(),
        config_path.join("paper.yml").to_string_lossy().to_string(),
        "--spigot-settings".to_string(),
        config_path.join("spigot.yml").to_string_lossy().to_string(),
        "--bukkit-settings".to_string(),
        config_path.join("bukkit.yml").to_string_lossy().to_string(),
        "--config".to_string(),
        config_path
            .join("server.properties")
            .to_string_lossy()
            .to_string(),
        "--commands-settings".to_string(),
        config_path
            .join("commands.yml")
            .to_string_lossy()
            .to_string(),
        // Data files
        "--universe".to_string(),
        world_path.to_string_lossy().to_string(),
        "--plugins".to_string(),
        plugins_path.to_string_lossy().to_string(),
    ])
}
