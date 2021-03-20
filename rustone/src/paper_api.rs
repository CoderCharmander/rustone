use crate::{
    config::{MinecraftVersion, ServerVersion},
    errors::*,
};
use log::{self, info};
use serde::Deserialize;
use std::fmt::{Display, Formatter};
use std::io;

#[derive(Deserialize, Debug)]
pub struct ProjectVersionList {
    pub project: String,
    pub versions: Vec<String>,
}

#[derive(Deserialize)]
pub struct PatchList {
    pub latest: u32,
    // all: Vec<u32>,
}

#[derive(Deserialize)]
struct PatchListResponse {
    builds: PatchList,
}

impl ProjectVersionList {
    pub async fn fetch(project: &str) -> Result<ProjectVersionList> {
        let url = "https://papermc.io/api/v1/".to_owned() + project;
        let resp = reqwest::get(&url)
            .await
            .chain_err(|| "failed to fetch versions")?
            .json()
            .await
            .chain_err(|| format!("failed to fetch versions for {}", project))?;
        Ok(resp)
    }

    pub async fn fetch_patches(version: MinecraftVersion, project: &str) -> Result<PatchList> {
        let url = format!("https://papermc.io/api/v1/{}/{}", project, version);
        let resp: PatchListResponse = reqwest::get(&url)
            .await
            .chain_err(|| "failed to connect to server")?
            .json()
            .await
            .chain_err(|| format!("failed to deserialize build list for version (this is likely a server-side problem) {:?}", version))?;
        Ok(resp.builds)
    }

    /// Download a server jar with the specified version into `stream`.
    pub async fn download<T: io::Write>(version: &mut ServerVersion, stream: &mut T) -> Result<()> {
        let (url, patch) = get_download_url(version).await?;
        info!("Downloading {}", url);
        version.patch = Some(patch);
        let resp = reqwest::get(&url)
            .await
            .chain_err(|| "failed to request jar from server")?
            .bytes()
            .await
            .chain_err(|| "failed to get raw binary jar")?;
        let mut cursor = io::Cursor::new(resp);
        io::copy(&mut cursor, stream).chain_err(|| "could not download jar")?;
        Ok(())
    }
}

async fn get_download_url(version: &ServerVersion) -> Result<(String, u32)> {
    let patch = match version.patch {
        Some(p) => Ok(p),
        None => ProjectVersionList::fetch_patches(version.minecraft, "paper")
            .await
            .map(|pl| pl.latest),
    }?;
    Ok((
        format!(
            "https://papermc.io/api/v1/paper/{}/{}/download",
            version.minecraft, patch
        ),
        patch,
    ))
}

impl Display for ProjectVersionList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.project, self.versions)
    }
}
