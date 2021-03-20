use std::{collections::HashMap, fs, io::Write, path::PathBuf};

use crate::config::MinecraftVersion;
use crate::{config::ServerVersion, errors::*, global::project_dirs};
use fs::File;
use lazy_static::lazy_static;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Structure representing a cached server jar file
pub struct CachedJar {
    pub version: ServerVersion,
}

#[derive(PartialEq, Hash, Eq, Clone)]
pub struct CachedJarMetaKey {
    pub version: MinecraftVersion,
    pub kind: String,
}

impl CachedJarMetaKey {
    pub fn path(&self) -> PathBuf {
        project_dirs()
            .unwrap()
            .cache_dir()
            .join(format!("{}-{}.jar", self.kind, self.version))
    }
}

impl Serialize for CachedJarMetaKey {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}@{}", self.kind, self.version))
    }
}

impl<'de> Deserialize<'de> for CachedJarMetaKey {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let str = <&str>::deserialize(deserializer)?;
        let mut parts = str.splitn(2, '@');
        Ok(Self {
            kind: parts.next().unwrap().to_owned(),
            version: parts
                .next()
                .ok_or_else(|| D::Error::custom("cache key must contain a '@'"))?
                .parse()
                .map_err(D::Error::custom)?,
        })
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CacheMeta {
    pub jars: HashMap<CachedJarMetaKey, u32>,
}

lazy_static! {
    static ref CACHE_META_PATH: PathBuf = project_dirs().unwrap().cache_dir().join("cache.toml");
}

pub fn get_cached(key: CachedJarMetaKey) -> Result<Option<CachedJar>> {
    let meta = read_cache_meta()?.jars;
    let patch = match meta.get(&key) {
        Some(p) => *p,
        None => return Ok(None),
    };
    let path = key.path();

    if !(path.exists() && path.is_file()) {
        return Ok(None);
    }
    Ok(Some(CachedJar {
        version: ServerVersion {
            minecraft: key.version,
            patch: Some(patch),
        },
    }))
}

pub fn get_cached_patch(key: &CachedJarMetaKey) -> Result<Option<u32>> {
    Ok(read_cache_meta()?.jars.get(&key).copied())
}

pub fn cache_jar(
    version: MinecraftVersion,
    patch: u32,
    kind: String,
) -> Result<(fs::File, CachedJar)> {
    let mut meta = read_cache_meta()?.jars;
    let key = CachedJarMetaKey { version, kind };
    if let Some(meta) = meta.get_mut(&key) {
        *meta = patch;
    } else {
        meta.insert(key.clone(), patch);
    }
    write_cache_meta(&CacheMeta { jars: meta })?;
    Ok((
        fs::File::create(key.path()).chain_err(|| "failed to cache file")?,
        CachedJar {
            version: ServerVersion {
                minecraft: version,
                patch: Some(patch),
            },
        },
    ))
}

pub fn read_cache_meta() -> Result<CacheMeta> {
    if !CACHE_META_PATH.exists() {
        return Ok(CacheMeta {
            jars: HashMap::new(),
        });
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

pub fn erase_cache() -> Result<()> {
    write_cache_meta(&CacheMeta {
        jars: HashMap::new(),
    })
}
