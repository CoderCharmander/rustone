extern crate toml;
use std::fmt::Display;

use crate::errors::*;
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
    pub extra_java_args: Vec<String>,
    pub extra_server_args: Vec<String>,
}

impl ServerVersion {
    pub fn new(data: &str) -> Result<Self> {
        let mut splitted = data.split(|c| c == '-' || c == '.');

        let minecraft = (
            splitted
                .next()
                .unwrap()
                .parse::<u32>()
                .chain_err(|| "Parse error for Minecraft major version")?,
            splitted
                .next()
                .ok_or(Error::from("No Minecraft minor version"))?
                .parse::<u32>()
                .chain_err(|| "Parse error for Minecraft minor version")?,
            splitted
                .next()
                .ok_or(Error::from("No Minecraft patch version"))?
                .parse::<u32>()
                .chain_err(|| "Parse error for Minecraft patch version")?,
        );
        let patch;
        if let Some(patch_str) = splitted.next() {
            patch = Some(
                patch_str
                    .parse::<u32>()
                    .chain_err(|| "Parse error for patch version")?,
            );
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
    pub fn new(str_config: &str) -> Result<Self> {
        let value: Value = str_config
            .parse::<Value>()
            .chain_err(|| "Parse error while loading config")?;

        let name = value
            .get("name")
            .ok_or(Error::from("'name' property does not exist"))?
            .as_str()
            .ok_or(Error::from("'name' is not a string"))?
            .to_string();

        let version = value
            .get("version")
            .ok_or(Error::from("'version' property does not exist"))?
            .as_str()
            .ok_or(Error::from("'version' is not a string"))?;

        // let extra_java_args = value
        //     .get("extra_java_args")
        //     .map_or_else(
        //         || Ok(vec![]),
        //         |v| {
        //             v.as_array()
        //                 .ok_or(Error::from("'extra_java_args' is not a list"))
        //         },
        //     )?
        //     .iter()
        //     .map(|v| v.as_str());

        // if extra_java_args.any(|s| s.is_none()) {
        //     return Err(Error::from("'extra_java_args' is not a list of strings"));
        // }
        // let extra_java_args = extra_java_args.map(|s| s.unwrap().to_string()).collect();

        // let extra_server_args = value
        //     .get("extra_server_args")
        //     .map_or_else(
        //         || Ok(&vec![]),
        //         |v| {
        //             v.as_array()
        //                 .ok_or(Error::from("'extra_server_args' is not a list"))
        //         },
        //     )?
        //     .iter()
        //     .map(|v| v.as_str());

        // if extra_server_args.any(|s| s.is_none()) {
        //     return Err(Error::from("'extra_server_args' is not a list of strings"));
        // }
        // let extra_server_args = extra_server_args.map(|s| s.unwrap().to_string()).collect();

        Ok(Self {
            version: ServerVersion::new(version)?,
            name,
            extra_java_args: vec![],
            extra_server_args: vec![],
        })
    }
}
