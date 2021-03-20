use std::io::Write;

use indicatif::MultiProgress;
use rustone::{
    cacher::{cache_jar, erase_cache, read_cache_meta},
    config::ServerVersion,
    server_kinds::ServerKind,
};

use rustone::errors::*;

pub fn purge() -> Result<()> {
    println!("Purging cache...");
    let meta = read_cache_meta()?.jars;
    let pb = crate::cli::create_progressbar(meta.len() as u64);
    for (jar, _) in meta {
        std::fs::remove_file(jar.path()).chain_err(|| "failed to delete")?;
        pb.inc(1);
    }
    erase_cache()?;
    pb.finish_with_message("done");
    Ok(())
}

pub async fn upgrade() -> Result<()> {
    println!("Upgrading jars...");
    let mut handles = vec![];
    let multibar = MultiProgress::new();
    for (cjmk, patch) in read_cache_meta()?.jars {
        let kind = cjmk.kind.parse::<ServerKind>()?;
        let latest_patch = kind.get_latest_patch(&cjmk.version).await?;
        if latest_patch > patch {
            let mut resp = kind
                .download_response(&mut ServerVersion {
                    minecraft: cjmk.version,
                    patch: Some(latest_patch),
                })
                .await?;
            let (mut out_file, _) = cache_jar(cjmk.version, latest_patch, cjmk.kind)?;
            let pb = crate::cli::create_download_progressbar(resp.content_length());
            let pb = multibar.add(pb);
            handles.push(tokio::spawn(async move {
                while let Some(chunk) = resp
                    .chunk()
                    .await
                    .chain_err(|| "failed to chunk response")?
                {
                    out_file
                        .write_all(&chunk)
                        .chain_err(|| "failed to write into file")?;
                    pb.inc(chunk.len() as u64);
                }
                Ok(()) as Result<()>
            }));
        }
    }
    multibar
        .join()
        .chain_err(|| "failed to join the progress bars")?;
    if !handles.is_empty() {
        let merged = futures::future::join_all(handles);
        let result = merged.await;
        for r in result {
            if r.is_err() {
                return Err("failed to join an upgrade task".into());
            } else {
                r.unwrap()?; // return an error if a future returned one
            }
        }
    } else {
        println!("no work to do");
    }
    Ok(())
}
