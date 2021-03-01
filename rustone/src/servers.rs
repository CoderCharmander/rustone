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

pub fn server_args(server: Server) -> Result<Vec<String>> {
    let config_path = server
        .server_path()?
        .join("configs")
        .canonicalize()
        .chain_err(|| "canonicalize failed")?;
    let world_path = server
        .server_path()?
        .join("worlds")
        .canonicalize()
        .chain_err(|| "canonicalize failed")?;
    let plugins_path = server
        .server_path()?
        .join("plugins")
        .canonicalize()
        .chain_err(|| "canonicalize failed")?;

    Ok(vec![
        // Don't open a GUI, that could interfere with us
        "--nogui".to_string(),
        // Config files
        "--paper-settings".to_string(),
        config_path.join("paper.yml").to_string_lossy().to_string(),
        "--spigot-settings".to_string(),
        config_path.join("spigot.yml").to_string_lossy().to_string(),
        "--bukkit-settings".to_string(),
        config_path.join("bukkit.yml").to_string_lossy().to_string(),
        "--config".to_string(),
        config_path
            .join("server.properties")
            .to_string_lossy()
            .to_string(),
        "--commands-settings".to_string(),
        config_path
            .join("commands.yml")
            .to_string_lossy()
            .to_string(),
        // Data files
        "--universe".to_string(),
        world_path.to_string_lossy().to_string(),
        "--plugins".to_string(),
        plugins_path.to_string_lossy().to_string(),
    ])
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

    pub fn create(name: &str, version: ServerVersion) -> Result<Self> {
        let mut file = fs::File::create(CONFIG_SERVER_DIR.join(format!("{}.toml", name)))
            .chain_err(|| "could not create server config")?;

        for dir in &["configs", "worlds", "plugins"] {
            fs::create_dir_all(DATA_SERVER_DIR.join(name).join(dir))
                .chain_err(|| format!("could not create directory {}", dir))?;
        }

        write!(&mut file, "name = '{}'\nversion = '{}'\n", name, version)
            .chain_err(|| "failed to write config file")?;

        let mut eula =
            fs::File::create(DATA_SERVER_DIR.join(name).join("configs").join("eula.txt"))
                .chain_err(|| "could not create eula file")?;
        writeln!(&mut eula, "eula=true").chain_err(|| "could not write eula file")?;

        Ok(Self {
            config: ServerConfig {
                name: name.to_owned(),
                version,
                extra_java_args: vec![],
                extra_server_args: vec![],
            },
        })
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
pub fn iter_servers_directory() -> Result<ReadDir> {
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
