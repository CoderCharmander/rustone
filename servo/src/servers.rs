use directories::ProjectDirs;
use fs::{read_dir, ReadDir};

use crate::{
    config::{ServerConfig, ServerVersion},
    errors::*,
    paper_api,
};
use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Child, Command, Stdio},
};

pub struct Server {
    pub config: ServerConfig,
}

fn project_dirs() -> Result<ProjectDirs> {
    directories::ProjectDirs::from("org", "CoderCharmander", "Servo")
        .ok_or("could not get directory information".into())
}

impl Server {
    pub fn get(name: &str) -> Result<Self> {
        let config_str =
            fs::read_to_string(project_dirs()?.config_dir().join(format!("{}.toml", name)))
                .chain_err(|| "could not load config file")?;
        let config = ServerConfig::new(&config_str)?;
        Ok(Self { config })
    }

    pub fn create(name: &str, version: ServerVersion) -> Result<Self> {
        let mut file =
            fs::File::create(project_dirs()?.config_dir().join(format!("{}.toml", name)))
                .chain_err(|| "could not create server config")?;

        for dir in &["configs", "worlds", "plugins"] {
            fs::create_dir_all(project_dirs()?.data_dir().join(name).join(dir))
                .chain_err(move || format!("could not create directory {}", dir))?;
        }

        write!(&mut file, "name = '{}'\nversion = '{}'\n", name, version)
            .chain_err(|| "failed to write config file")?;

        let mut eula = fs::File::create(
            project_dirs()?
                .data_dir()
                .join(name)
                .join("configs")
                .join("eula.txt"),
        )
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
        let project_dirs = directories::ProjectDirs::from("org", "CoderCharmander", "Servo")
            .ok_or(Error::from("Could not create directories!"))?;
        if !project_dirs.config_dir().exists() {
            fs::create_dir_all(project_dirs.config_dir())
                .chain_err(|| "could not create config directory")?;
        }

        let config = project_dirs.config_dir().join(format!(
            "{}.toml",
            project_dirs.config_dir().join(&self.config.name).display()
        ));

        Ok(config)
    }

    pub fn server_path(&self) -> Result<PathBuf> {
        let project_dirs = directories::ProjectDirs::from("org", "CoderCharmander", "Servo")
            .ok_or(Error::from("Could not create directories!"))?;
        let path = project_dirs.data_dir().join(&self.config.name);
        if !path.exists() {
            fs::create_dir_all(project_dirs.data_dir().join(&self.config.name))
                .chain_err(|| "could not create server directory")?;
        }
        Ok(path)
    }
}

/// Structure representing a cached server jar file
pub struct CachedJar {
    pub version: ServerVersion,
    pub path: PathBuf,
}

impl CachedJar {
    /// Download the jar file corresponding to `version` and create a `CachedJar`.
    pub fn download(version: ServerVersion) -> Result<Self> {
        let path = project_dirs()?
            .cache_dir()
            .join(format!("paper-{}.jar", version));
        fs::create_dir_all(project_dirs()?.cache_dir())
            .chain_err(|| "could not create cache directory")?;
        if path.is_file() {
            return Ok(Self { path, version });
        }
        println!("Downloading server jar version {}", version);
        let mut file =
            fs::File::create(path).chain_err(|| "could not create jar file to download")?;
        paper_api::ProjectVersionList::download(&version, &mut file)
            .chain_err(|| "could not download server jar")?;
        Ok(Self {
            path: project_dirs()?
                .cache_dir()
                .join(format!("paper-{}.jar", version)),
            version,
        })
    }

    /// Start the server with the cached jar.
    pub fn start_server(
        &self,
        server: Server,
        stdout: Stdio,
        stderr: Stdio,
        stdin: Stdio,
    ) -> Result<Child> {
        if server.config.version != self.version {
            return Err(Error::from("Server started with invalid version!"));
        }

        println!("Starting server {}", server.config.name);

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

        let cmd = Command::new("java")
            .args(&[
                "-jar",
                self.path
                    .canonicalize()
                    .chain_err(|| "canonicalize failed")?
                    .to_str()
                    .unwrap(),
                // Don't open a GUI, that could interfere with us
                "--nogui",
                // Config files
                "--paper-settings",
                config_path.join("paper.yml").to_str().unwrap(),
                "--spigot-settings",
                config_path.join("spigot.yml").to_str().unwrap(),
                "--bukkit-settings",
                config_path.join("bukkit.yml").to_str().unwrap(),
                "--config",
                config_path.join("server.properties").to_str().unwrap(),
                "--commands-settings",
                config_path.join("commands.yml").to_str().unwrap(),
                // Data files
                "--universe",
                world_path.to_str().unwrap(),
                "-plugins",
                plugins_path.to_str().unwrap(),
            ])
            .current_dir(config_path)
            .stdout(stdout)
            .stderr(stderr)
            .stdin(stdin)
            .spawn()
            .chain_err(|| "spawning the server process failed")?;
        Ok(cmd)
    }
}

/// Return an iterator to the servers directory.
pub fn iter_servers_directory() -> Result<ReadDir> {
    read_dir(project_dirs()?.data_dir()).chain_err(|| "failed to list servers directory")
}

pub fn iter_servers() -> Result<impl Iterator<Item = Result<Server>>> {
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
    Ok(mapped)
}
