use crate::config::ServerVersion;
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
    pub async fn fetch(project: &str) -> Result<ProjectVersionList, Box<dyn std::error::Error>> {
        let url = "https://papermc.io/api/v1/".to_owned() + project;
        let resp = reqwest::get(&url)
            .await?
            .json::<ProjectVersionList>()
            .await?;
        Ok(resp)
    }

    pub async fn fetch_patches(
        version: (u32, u32, u32),
    ) -> Result<PatchList, Box<dyn std::error::Error>> {
        let url = format!(
            "https://papermc.io/api/v1/paper/{}.{}.{}",
            version.0, version.1, version.2
        );
        let resp = reqwest::get(&url)
            .await?
            .json::<PatchListResponse>()
            .await?;
        Ok(resp.builds)
    }

    pub async fn download<T: io::Write>(
        version: ServerVersion,
        stream: &mut T,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let url;
        if let Some(patch) = version.patch {
            url = format!(
                "https://papermc.io/api/v1/paper/{}.{}.{}/{}/download",
                version.minecraft.0, version.minecraft.1, version.minecraft.2, patch
            );
        } else {
            url = format!(
                "https://papermc.io/api/v1/paper/{}.{}.{}/{}/download",
                version.minecraft.0,
                version.minecraft.1,
                version.minecraft.2,
                Self::fetch_patches(version.minecraft).await?.latest
            );
        }
        info!("Downloading {}", url);
        let resp = reqwest::get(&url).await?.bytes().await?;
        io::copy(&mut resp.as_ref(), stream)?;
        Ok(())
    }
}

impl Display for ProjectVersionList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.project, self.versions)
    }
}
