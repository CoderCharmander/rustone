use std::{process::Stdio, str::FromStr};

use crate::{
    config::{MinecraftVersion, ServerConfig, ServerVersion},
    errors::*,
};
use error_chain::bail;
use reqwest::Response;

pub mod forge;
pub mod paper;

pub enum ServerKind {
    Paper,
}

impl ServerKind {
    pub async fn is_latest_patch(&self, version: &MinecraftVersion, patch: u32) -> Result<bool> {
        match self {
            Self::Paper => paper::is_latest_patch(version, patch).await,
        }
    }

    pub async fn get_latest(&self) -> Result<ServerVersion> {
        match self {
            Self::Paper => paper::get_latest().await,
        }
    }

    pub async fn get_latest_patch(&self, version: &MinecraftVersion) -> Result<u32> {
        match self {
            Self::Paper => paper::get_latest_patch(version).await,
        }
    }

    pub async fn download_response(&self, version: &mut ServerVersion) -> Result<Response> {
        match self {
            Self::Paper => paper::download_response(version).await,
        }
    }

    pub fn initialize(&self, config: &ServerConfig) -> Result<()> {
        match self {
            Self::Paper => paper::initialize(config),
        }
    }

    pub fn launch(
        &self,
        config: ServerConfig,
        stdout: Stdio,
        stdin: Stdio,
        stderr: Stdio,
    ) -> Result<tokio::process::Child> {
        match self {
            Self::Paper => paper::launch(config, stdout, stdin, stderr),
        }
    }
}

impl FromStr for ServerKind {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "paper" => Ok(Self::Paper),
            _ => bail!("string {} is not a valid server type", s),
        }
    }
}

impl ToString for ServerKind {
    fn to_string(&self) -> String {
        match self {
            Self::Paper => "paper".to_owned(),
        }
    }
}
