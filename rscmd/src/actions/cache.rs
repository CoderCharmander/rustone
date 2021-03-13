use rustone::{cacher::{CachedJar, purge_jar, read_cache_meta}, config::ServerVersion};

use rustone::errors::*;

pub fn purge() -> Result<()> {
    let meta = read_cache_meta()?.jars;
    for jar in meta {
        purge_jar(jar.mcversion)?;
    }
    Ok(())
}

pub fn upgrade() -> Result<()> {
    println!("Upgrading jars...");
    for cj in read_cache_meta()?.jars {
        println!("Conditionally upgrading version {}...", cj.mcversion);
        CachedJar::get(ServerVersion {
            minecraft: cj.mcversion,
            patch: None,
        })?;
    }
    Ok(())
}