use std::{fs, io::Write};
use indicatif::ProgressBar;
use rustone::errors::*;
use crate::cli;

pub async fn download(response: &mut reqwest::Response, file: &mut fs::File) -> Result<()> {
    let pb = cli::create_download_progressbar(response.content_length());
    download_with_pb(response, file, &pb).await
}

pub async fn download_with_pb(response: &mut reqwest::Response, file: &mut fs::File, pb: &ProgressBar) -> Result<()> {
    while let Some(chunk) = response
        .chunk()
        .await
        .chain_err(|| "failed to read chunk")?
    {
        file.write_all(&chunk)
            .chain_err(|| "failed to write into file")?;
        pb.inc(chunk.len() as u64);
    }
    pb.finish_with_message("done");
    Ok(())
}