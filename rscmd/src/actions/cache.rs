use rustone::{
    cacher::{purge_jar, read_cache_meta, CachedJar},
    config::ServerVersion,
};

use rustone::errors::*;

pub fn purge() -> Result<()> {
    let meta = read_cache_meta()?.jars;
    for jar in meta {
        purge_jar(jar.mcversion)?;
    }
    Ok(())
}

pub async fn upgrade() -> Result<()> {
    println!("Upgrading jars...");
    let mut handles = vec![];
    for cj in read_cache_meta()?.jars {
        println!("Conditionally upgrading version {}...", cj.mcversion);

        let future = CachedJar::get(ServerVersion {
            minecraft: cj.mcversion,
            patch: None,
        });

        handles.push(tokio::spawn(future));
    }
    let merged = futures::future::join_all(handles);
    let result = merged.await;
    for r in result {
        if let Err(_) = r {
            return Err("failed to join an upgrade task".into());
        } else {
            r.unwrap()?; // return an error if a future returned one
        }
    }
    Ok(())
}
