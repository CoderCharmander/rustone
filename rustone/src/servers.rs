use crate::server_kinds::ServerKind;
use fs::{read_dir, ReadDir};
use lazy_static::lazy_static;

use crate::{
    config::{ServerConfig, ServerVersion},
    errors::*,
    global::*,
};
use std::{fs, io::Write, path::PathBuf};

pub struct Server {
    pub config: ServerConfig,
}

lazy_static! {
    pub static ref CONFIG_SERVER_DIR: PathBuf =
        project_dirs().unwrap().config_dir().join("servers");
    pub static ref DATA_SERVER_DIR: PathBuf = project_dirs().unwrap().data_dir().into();
}

impl ServerConfig {
    pub fn path(&self) -> PathBuf {
        DATA_SERVER_DIR.join(&self.name)
    }

    pub fn config_path(&self) -> PathBuf {
        CONFIG_SERVER_DIR.join(format!("{}.toml", self.name))
    }
}

impl Server {
    pub fn get(name: &str) -> Result<Self> {
        let path = project_dirs()?
            .config_dir()
            .join("servers")
            .join(format!("{}.toml", name));
        let config_str = fs::read_to_string(
            project_dirs()?
                .config_dir()
                .join("servers")
                .join(format!("{}.toml", name)),
        )
        .chain_err(|| format!("could not load config file: {}", path.to_string_lossy()))?;
        let config = ServerConfig::new(&config_str)?;
        Ok(Self { config })
    }

    pub fn create(name: &str, version: ServerVersion, kind: String) -> Result<Self> {
        let mut file = fs::File::create(CONFIG_SERVER_DIR.join(format!("{}.toml", name)))
            .chain_err(|| "could not create server config")?;

        let server_kind = kind.parse::<ServerKind>()?;

        let config = ServerConfig {
            name: name.to_owned(),
            version,
            extra_java_args: vec![],
            extra_server_args: vec![],
            kind,
        };

        server_kind.initialize(&config)?;

        file.write_all(
            toml::to_string_pretty(&config)
                .chain_err(|| "failed to generate configuration")?
                .as_bytes(),
        )
        .chain_err(|| "failed to write config file")?;

        let mut eula =
            fs::File::create(DATA_SERVER_DIR.join(name).join("configs").join("eula.txt"))
                .chain_err(|| "could not create eula file")?;
        writeln!(&mut eula, "eula=true").chain_err(|| "could not write eula file")?;

        Ok(Self { config })
    }

    pub fn config_path(&self) -> Result<PathBuf> {
        if !CONFIG_SERVER_DIR.exists() {
            fs::create_dir_all(CONFIG_SERVER_DIR.clone().into_boxed_path())
                .chain_err(|| "could not create config directory")?;
        }

        let config = CONFIG_SERVER_DIR.join(format!("{}.toml", self.config.name,));
        Ok(config)
    }

    pub fn server_path(&self) -> Result<PathBuf> {
        let path = DATA_SERVER_DIR.join(&self.config.name);
        if !path.exists() {
            fs::create_dir_all(path.clone()).chain_err(|| "could not create server directory")?;
        }
        Ok(path)
    }
}

/// Return an iterator to the servers directory.
fn iter_servers_directory() -> Result<ReadDir> {
    read_dir(project_dirs()?.data_dir()).chain_err(|| "failed to list servers directory")
}

pub fn get_servers() -> Result<Vec<Server>> {
    let dir = iter_servers_directory()?;
    let mapped = dir.map(|entry| {
        Server::get(
            entry
                .chain_err(|| "failed to read directory entry")?
                .file_name()
                .to_str()
                .unwrap(),
        )
    });
    Ok(mapped.collect::<Result<_>>()?)
}
