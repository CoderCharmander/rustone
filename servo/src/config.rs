extern crate toml;
use crate::error::ServoError;
use toml::Value;

#[derive(Debug)]
pub struct ServerVersion {
    pub minecraft: (u32, u32, u32),
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
            splitted
                .next()
                .ok_or(ServoError::boxnew("what"))?
                .parse::<u32>()?,
            splitted
                .next()
                .ok_or(ServoError::boxnew("No Minecraft minor version"))?
                .parse::<u32>()?,
            splitted
                .next()
                .ok_or(ServoError::boxnew("No Minecraft patch version"))?
                .parse::<u32>()?,
        );
        let patch ;
        if let Some(patch_str) = splitted.next() {
            patch = Some(patch_str.parse::<u32>()?);
        } else {
            patch = None;
        }

        Ok(Self {
            minecraft,
            // If the patch version is 0, it means "just get the latest"
            patch,
        })
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
