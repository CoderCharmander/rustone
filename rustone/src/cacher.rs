use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
};

use fs::File;
use lazy_static::lazy_static;

use crate::config::MinecraftVersion;
use crate::{
    config::ServerVersion,
    errors::*,
    global::project_dirs,
    paper_api,
    servers::{server_args, Server},
};

/// Structure representing a cached server jar file
pub struct CachedJar {
    pub version: ServerVersion,
    path: PathBuf,
}

impl CachedJar {
    pub fn get(version: ServerVersion) -> Result<Self> {
        if is_cached_jar_latest(version.minecraft) {
            return Ok(Self {
                path: cached_jar_path(version.minecraft),
                version,
            });
        }
        let out = Self::download(version)?;
        add_jar_to_cache_meta(CachedJarMeta {
            mcversion: version.minecraft,
            patch: out.version.patch.unwrap(),
        })?;
        Ok(out)
    }

    /// Download the jar file corresponding to `version` and create a `CachedJar`.
    pub fn download(mut version: ServerVersion) -> Result<Self> {
        let path = project_dirs()?
            .cache_dir()
            .join(format!("paper-{}.jar", version));
        fs::create_dir_all(project_dirs()?.cache_dir())
            .chain_err(|| "could not create cache directory")?;
        println!("Downloading server jar version {}", version);
        let mut file =
            fs::File::create(path).chain_err(|| "could not create jar file to download")?;
        paper_api::ProjectVersionList::download(&mut version, &mut file)
            .chain_err(|| "could not download server jar")?;
        Ok(Self {
            path: project_dirs()?
                .cache_dir()
                .join(format!("paper-{}.jar", version.minecraft)),
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

        let cmd = Command::new("java")
            .args(&["-jar", &self.path.to_string_lossy()])
            .args(server_args(server)?)
            // There might be other config files, caches etc. We don't want them
            // to clutter the current directory, even less to be lost or overridden
            // accidentally.
            .current_dir(config_path)
            .stdout(stdout)
            .stderr(stderr)
            .stdin(stdin)
            .spawn()
            .chain_err(|| "spawning the server process failed")?;
        Ok(cmd)
    }

    fn get_cached_path(version: MinecraftVersion) -> PathBuf {
        project_dirs()
            .unwrap()
            .cache_dir()
            .join(format!("paper-{}.jar", version))
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CachedJarMeta {
    pub mcversion: MinecraftVersion,
    pub patch: u32,
}
#[derive(serde::Serialize, serde::Deserialize)]
pub struct CacheMeta {
    pub jars: Vec<CachedJarMeta>,
}

lazy_static! {
    static ref CACHE_META_PATH: PathBuf = project_dirs().unwrap().cache_dir().join("cache.toml");
}

pub fn read_cache_meta() -> Result<CacheMeta> {
    if !CACHE_META_PATH.exists() {
        return Ok(CacheMeta { jars: vec![] });
    }
    let text = fs::read_to_string(CACHE_META_PATH.clone().into_os_string())
        .chain_err(|| "failed to read cache metadata")?;
    toml::from_str(&text).chain_err(|| "failed to parse cache metadata")
}

fn write_cache_meta(meta: &CacheMeta) -> Result<()> {
    let ser = toml::to_string(meta).chain_err(|| "failed to serialize cache metadata")?;
    let mut file = File::create(CACHE_META_PATH.clone().into_os_string())
        .chain_err(|| "failed to create cache metadata file")?;
    file.write_all(ser.as_bytes())
        .chain_err(|| "failed to write into cache metadata file")?;
    Ok(())
}

fn add_jar_to_cache_meta(meta: CachedJarMeta) -> Result<()> {
    let mut cur_meta = read_cache_meta()?;
    if let Some(cache_item_ref) = cur_meta
        .jars
        .iter_mut()
        .find(|c| c.mcversion == meta.mcversion)
    {
        cache_item_ref.patch = meta.patch;
    } else {
        cur_meta.jars.push(meta);
    }

    write_cache_meta(&cur_meta)?;

    Ok(())
}

fn cached_jar_path(version: MinecraftVersion) -> PathBuf {
    project_dirs()
        .unwrap()
        .cache_dir()
        .join(format!("paper-{}.jar", version))
}

pub fn is_cached_jar_available(version: MinecraftVersion) -> bool {
    project_dirs().map_or(false, |p| {
        Path::new(&p.cache_dir().join(format!("paper-{}.jar", version))).exists()
    })
}

pub fn get_cached_jar_patch(version: MinecraftVersion) -> Option<u32> {
    if !is_cached_jar_available(version) {
        return None;
    }
    let meta = read_cache_meta();
    let meta = match meta {
        Ok(meta) => meta,
        Err(_) => return None,
    };
    // Is there any cache entry matching the version?
    meta.jars
        .iter()
        .find(|c| c.mcversion == version)
        .map(|c| c.patch)
}

pub fn is_cached_jar_latest(version: MinecraftVersion) -> bool {
    let patch = get_cached_jar_patch(version);
    if patch.is_none() {
        return false;
    }
    let patch = patch.unwrap();
    // If we can't fetch the patches, we likely can't upgrade either
    match paper_api::ProjectVersionList::fetch_patches(version, "paper") {
        Ok(list) => patch >= list.latest,
        Err(_) => false,
    }
}

pub fn upgrade_jar(version: MinecraftVersion) -> Result<()> {
    if is_cached_jar_latest(version) {
        return Ok(());
    }
    let new_patch = paper_api::ProjectVersionList::fetch_patches(version, "paper")?.latest;
    let mut out_file = File::create(CachedJar::get_cached_path(version))
        .chain_err(|| "failed to create upgraded jar file")?;
    paper_api::ProjectVersionList::download(
        &mut ServerVersion {
            minecraft: version,
            patch: Some(new_patch),
        },
        &mut out_file,
    )?;
    add_jar_to_cache_meta(CachedJarMeta {
        mcversion: version,
        patch: new_patch,
    })
}
