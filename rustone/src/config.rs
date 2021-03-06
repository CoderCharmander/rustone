extern crate toml;
use std::{fmt::Display, num::ParseIntError, result, str::FromStr};

use crate::errors::*;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

/// Represents a Minecraft version
/// # Examples
/// ```
/// use crate::rustone::config::MinecraftVersion;
/// let ver = "1.12.2".parse::<MinecraftVersion>().unwrap();
/// assert_eq!(ver, MinecraftVersion(1, 12, Some(2)))
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinecraftVersion(pub u32, pub u32, pub Option<u32>);

impl<'a> MinecraftVersion {
    fn from_splitted<I>(splitted: &mut I) -> result::Result<Self, ParseIntError>
    where
        I: Iterator<Item = &'a str>,
    {
        let minecraft = (
            splitted.next().unwrap().parse()?,
            splitted.next().unwrap_or("").parse()?,
        );
        Ok(Self(
            minecraft.0,
            minecraft.1,
            match splitted.next() {
                Some(s) => Some(s.parse()?),
                None => None,
            },
        ))
    }
}

impl Display for MinecraftVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.0, self.1)?;
        if let Some(patch) = self.2 {
            write!(f, ".{}", patch)?;
        }
        Ok(())
    }
}

impl FromStr for MinecraftVersion {
    type Err = ParseIntError;

    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        let mut splitted = s.split(&['-', '.'][..]);
        Self::from_splitted(&mut splitted)
    }
}

impl Serialize for MinecraftVersion {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for MinecraftVersion {
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MinecraftVersionVisitor;
        impl<'de> Visitor<'de> for MinecraftVersionVisitor {
            type Value = MinecraftVersion;
            fn expecting(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(fmt, "Minecraft (semantic) version number")
            }

            fn visit_str<E: serde::de::Error>(self, v: &str) -> result::Result<Self::Value, E> {
                let parsed = v.parse::<MinecraftVersion>();
                parsed.map_err(|e| E::custom(format!("version parse error: {}", e)))
            }
        }
        deserializer.deserialize_str(MinecraftVersionVisitor)
    }
}

/// # Examples
/// ```
/// let version = crate::rustone::config::ServerVersion::new("1.12.2-2").unwrap();
/// assert_eq!(toml::to_string(&version).unwrap(), "\"1.12.2-2\"");
/// ```
#[derive(Debug, Clone, Copy)]
pub struct ServerVersion {
    /// Minecraft version.
    /// The last number (patch) may be missing (for example: 1.17)
    ///
    /// # Examples
    /// ```
    /// let version = crate::rustone::config::ServerVersion::new("1.12.2").unwrap();
    /// assert_eq!(version.minecraft, crate::rustone::config::MinecraftVersion(1, 12, Some(2)));
    /// ```
    pub minecraft: MinecraftVersion,

    /// Build number.
    ///
    /// # Examples
    /// ```
    /// use crate::rustone::config::ServerVersion;
    /// let version = ServerVersion::new("1.12.2-2").unwrap();
    /// assert_eq!(version.patch, Some(2));
    /// ```
    pub patch: Option<u32>,
}

impl ServerVersion {
    pub fn new(data: &str) -> Result<Self> {
        let mut splitted = data.split(&['.', '-'][..]);
        let minecraft = MinecraftVersion::from_splitted(&mut splitted)
            .chain_err(|| "failed to parse Minecraft version")?;

        let patch;
        if let Some(patch_str) = splitted.next() {
            patch = Some(
                patch_str
                    .parse::<u32>()
                    .chain_err(|| "parse error for Paper patch version")?,
            );
        } else {
            patch = None;
        }

        Ok(Self { minecraft, patch })
    }
}

impl Display for ServerVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.minecraft)?;
        if let Some(patch) = self.patch {
            write!(f, "-{}", patch)?;
        }
        Ok(())
    }
}

impl std::cmp::PartialEq<ServerVersion> for ServerVersion {
    fn eq(&self, other: &ServerVersion) -> bool {
        self.minecraft == other.minecraft
    }
}

impl serde::Serialize for ServerVersion {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let txt = self.to_string();
        serializer.serialize_str(&txt)
    }
}

impl<'de> Deserialize<'de> for ServerVersion {
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ServerVersionVisitor;
        impl<'de> Visitor<'de> for ServerVersionVisitor {
            type Value = ServerVersion;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(
                    formatter,
                    "Minecraft version number (semantic) + optional dash-separated build number"
                )
            }

            fn visit_str<E>(self, v: &str) -> result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let ver = ServerVersion::new(v);
                ver.map_err(|e| E::custom(e.to_string()))
            }
        }
        deserializer.deserialize_str(ServerVersionVisitor)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ServerConfig {
    pub version: ServerVersion,
    pub name: String,
    #[serde(default)]
    pub extra_java_args: Vec<String>,
    #[serde(default)]
    pub extra_server_args: Vec<String>,
}

impl ServerConfig {
    pub fn new(str_config: &str) -> Result<Self> {
        Ok(toml::from_str(str_config).chain_err(|| "invalid server configuration")?)
    }
}
