extern crate toml;
use std::{fmt::Display, path::PathBuf};

use crate::error::ServoError;
use toml::Value;

#[derive(Debug, Clone, Copy)]
pub struct ServerVersion {
    /// Minecraft version.
    /// # Examples
    /// ```
    /// let version = crate::servo::config::ServerVersion::new("1.12.2").unwrap();
    /// assert_eq!(version.minecraft, (1, 12, 2));
    /// ```
    pub minecraft: (u32, u32, u32),

    /// Build number.
    pub patch: Option<u32>,
}

pub struct ServerConfig {
    pub version: ServerVersion,
    pub name: String,
}

impl ServerVersion {
    pub fn new(data: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut splitted = data.split(|c| c == '-' || c == '.');

        let minecraft = (
            splitted.next().unwrap().parse::<u32>()?,
            splitted
                .next()
                .ok_or(ServoError::boxnew("No Minecraft minor version"))?
                .parse::<u32>()?,
            splitted
                .next()
                .ok_or(ServoError::boxnew("No Minecraft patch version"))?
                .parse::<u32>()?,
        );
        let patch;
        if let Some(patch_str) = splitted.next() {
            patch = Some(patch_str.parse::<u32>()?);
        } else {
            patch = None;
        }

        Ok(Self { minecraft, patch })
    }
}

impl Display for ServerVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(patch) = self.patch {
            write!(
                f,
                "{}.{}.{}-{}",
                self.minecraft.0, self.minecraft.1, self.minecraft.2, patch
            )
        } else {
            write!(
                f,
                "{}.{}.{}",
                self.minecraft.0, self.minecraft.1, self.minecraft.2
            )
        }
    }
}

impl std::cmp::PartialEq<ServerVersion> for ServerVersion {
    fn eq(&self, other: &ServerVersion) -> bool {
        self.patch == other.patch && self.minecraft == other.minecraft
    }
}

impl ServerConfig {
    pub fn new(str_config: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let value: Value = str_config.parse::<Value>()?;

        let name = value
            .get("name")
            .ok_or(ServoError::boxnew("'name' property does not exist"))?
            .as_str()
            .ok_or(ServoError::boxnew("'name' is not a string"))?
            .to_string();

        let version = value
            .get("version")
            .ok_or(ServoError::boxnew("'version' property does not exist"))?
            .as_str()
            .ok_or(ServoError::boxnew("'version' is not a string"))?;

        Ok(Self {
            version: ServerVersion::new(version)?,
            name,
        })
    }
}
