use directories::ProjectDirs;

use crate::error::{Result, ServoError};
use crate::{
    config::{ServerConfig, ServerVersion},
    paper_api,
};
use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Child, Command},
};

pub struct Server {
    pub config: ServerConfig,
}

fn project_dirs() -> Result<ProjectDirs> {
    directories::ProjectDirs::from("org", "CoderCharmander", "Servo")
        .ok_or(ServoError::boxnew("Could not get directory information"))
}

impl Server {
    pub fn get(name: &str) -> Result<Self> {
        let config_str =
            fs::read_to_string(project_dirs()?.config_dir().join(format!("{}.toml", name)))?;
        let config = ServerConfig::new(&config_str)?;
        Ok(Self { config })
    }

    pub fn create(name: &str, version: ServerVersion) -> Result<Self> {
        let mut file =
            fs::File::create(project_dirs()?.config_dir().join(format!("{}.toml", name)))?;

        for dir in &["configs", "worlds", "plugins"] {
            fs::create_dir_all(project_dirs()?.data_dir().join(name).join(dir))?;
        }

        writeln!(&mut file, "name = '{}'", name)?;
        writeln!(&mut file, "version = '{}'", version)?;

        let mut eula = fs::File::create(
            project_dirs()?
                .data_dir()
                .join(name)
                .join("configs")
                .join("eula.txt"),
        )?;
        writeln!(&mut eula, "eula=true")?;

        Ok(Self {
            config: ServerConfig {
                name: name.to_owned(),
                version,
            },
        })
    }

    pub fn config_path(&self) -> Result<PathBuf> {
        let project_dirs = directories::ProjectDirs::from("org", "CoderCharmander", "Servo")
            .ok_or(ServoError::boxnew("Could not create directories!"))?;
        if !project_dirs.config_dir().exists() {
            fs::create_dir_all(project_dirs.config_dir())?;
        }

        let config = project_dirs.config_dir().join(format!(
            "{}.conf",
            project_dirs.config_dir().join(&self.config.name).display()
        ));

        Ok(config)
    }

    pub fn server_path(&self) -> Result<PathBuf> {
        let project_dirs = directories::ProjectDirs::from("org", "CoderCharmander", "Servo")
            .ok_or(ServoError::boxnew("Could not create directories!"))?;
        let path = project_dirs.data_dir().join(&self.config.name);
        if !path.exists() {
            fs::create_dir_all(project_dirs.data_dir().join(&self.config.name))?;
        }
        Ok(path)
    }
}

pub struct CachedJar {
    pub version: ServerVersion,
    pub path: PathBuf,
}

impl CachedJar {
    pub fn download(version: ServerVersion) -> Result<Self> {
        let path = project_dirs()?
            .cache_dir()
            .join(format!("paper-{}.jar", version));
        fs::create_dir_all(project_dirs()?.cache_dir())?;
        if path.is_file() {
            return Ok(Self { path, version });
        }
        println!("Downloading server jar version {}", version);
        let mut file = fs::File::create(path)?;
        paper_api::ProjectVersionList::download(&version, &mut file)?;
        Ok(Self {
            path: project_dirs()?
                .cache_dir()
                .join(format!("paper-{}.jar", version)),
            version,
        })
    }

    pub fn start_server(&self, server: Server) -> Result<Child> {
        if server.config.version != self.version {
            return Err(ServoError::boxnew("Server started with invalid version!"));
        }

        println!("Starting server {}", server.config.name);

        let config_path = server.server_path()?.join("configs").canonicalize()?;
        let world_path = server.server_path()?.join("worlds").canonicalize()?;
        let plugins_path = server.server_path()?.join("plugins").canonicalize()?;

        let cmd = Command::new("java")
            .args(&[
                "-jar",
                self.path.canonicalize()?.to_str().unwrap(),
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
            .spawn()?;
        Ok(cmd)
    }
}
