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
    latest: u32,
    // all: Vec<u32>,
}

#[derive(Deserialize)]
struct PatchListResponse {
    builds: PatchList,
}

impl ProjectVersionList {
    pub async fn fetch(project: &str) -> Result<ProjectVersionList> {
        let url = "https://papermc.io/api/v1/".to_owned() + project;
        let resp = ureq::get(&url)
            .call()
            .into_json_deserialize::<ProjectVersionList>()
            .chain_err(|| format!("failed to fetch versions for {}", project))?;
        Ok(resp)
    }

    pub fn fetch_patches(version: MinecraftVersion) -> Result<PatchList> {
        let url = format!("https://papermc.io/api/v1/paper/{}", version);
        let resp = ureq::get(&url)
            .call()
            .into_json_deserialize::<PatchListResponse>()
            .chain_err(|| format!("failed to request build list for version {:?}", version))?;
        Ok(resp.builds)
    }

    /// Download a server jar with the specified version into `stream`.
    pub fn download<T: io::Write>(version: &ServerVersion, stream: &mut T) -> Result<()> {
        let url = get_download_url(version)?;
        info!("Downloading {}", url);
        let mut resp = ureq::get(&url).call().into_reader();
        io::copy(&mut resp, stream).chain_err(|| "could not download jar")?;
        Ok(())
    }
}

fn get_download_url(version: &ServerVersion) -> Result<String> {
    Ok(format!(
        "https://papermc.io/api/v1/paper/{}/{}/download",
        version.minecraft,
        version.patch.map_or_else(
            || ProjectVersionList::fetch_patches(version.minecraft).map(|pl| pl.latest),
            |p| Ok(p)
        )?
    ))
}

impl Display for ProjectVersionList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.project, self.versions)
    }
}
