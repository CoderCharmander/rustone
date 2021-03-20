extern crate error_chain;
pub mod cacher;
pub mod config;
pub mod errors;
pub mod server_kinds;
pub mod servers;

pub mod global {
    use crate::errors;
    use directories::ProjectDirs;

    pub fn project_dirs() -> errors::Result<ProjectDirs> {
        directories::ProjectDirs::from("org", "CoderCharmander", "Rustone")
            .ok_or_else(|| "could not get directory information".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_parse() {
        let version = config::ServerVersion::new("1.12.2-4").unwrap();
        assert_eq!(version.patch.unwrap(), 4);
        assert_eq!(version.minecraft, config::MinecraftVersion(1, 12, Some(2)));
    }

    #[test]
    #[should_panic(expected = "ParseIntError")]
    fn incorrect_version_parse() {
        config::ServerVersion::new("blah.blah-blah").unwrap();
    }

    #[test]
    fn config_parse() {
        let config = config::ServerConfig::new("name = 'something'\nversion = '1.12.2-4'").unwrap();
        assert_eq!(config.name, "something");
        assert_eq!(
            config.version,
            config::ServerVersion::new("1.12.2-4").unwrap()
        );
    }

    #[test]
    #[should_panic(expected = "invalid server configuration")]
    fn incorrect_config_parse() {
        let config = config::ServerConfig::new("version = '1.12.2-4'").unwrap();
        assert_eq!(config.name, "something");
        assert_eq!(
            config.version,
            config::ServerVersion::new("1.12.2-4").unwrap()
        );
    }
}
