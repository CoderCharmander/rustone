use std::{io::Write, process::Stdio};

use http::StatusCode;
use json::JsonValue;
use rustone::{
    cacher::{self, CachedJarMetaKey},
    config::{MinecraftVersion, ServerConfig, ServerVersion},
    errors::{self, ResultExt},
    server_kinds::ServerKind,
    servers,
};
use warp::reply::with_status;

macro_rules! route_try {
    ($x:expr) => {{
        match $x {
            Ok(o) => o,
            Err(e) => {
                return Ok(with_status(
                    json::stringify(json::object! {success: false, payload: e.to_string()}),
                    StatusCode::INTERNAL_SERVER_ERROR,
                ))
            }
        }
    }};
}

pub async fn get_server(name: String) -> std::result::Result<impl warp::Reply, warp::Rejection> {
    let server = servers::Server::get(&name);
    match server {
        Ok(server) => Ok(warp::reply::with_status(
            json::stringify(json::object! {
                success: true,
                payload: {
                    version: format!("{}", server.config.version)
                }
            }),
            StatusCode::OK,
        )),
        Err(error) => Ok(warp::reply::with_status(
            json::stringify(json::object! {success: false, payload: error.to_string()}),
            StatusCode::NOT_FOUND,
        )),
    }
}

pub async fn list_servers() -> std::result::Result<impl warp::Reply, warp::Rejection> {
    let servers_iter = servers::get_servers();
    if let Err(error) = servers_iter {
        return Ok(warp::reply::with_status(
            json::stringify(json::object! {success: false, payload: error.to_string()}),
            StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }
    let servers: Vec<JsonValue> = servers_iter
        .unwrap()
        .iter()
        .map(|s| json::object! {name: s.config.name.clone(), version: format!("{}", s.config.version)})
        .collect();

    Ok(warp::reply::with_status(
        json::stringify(json::object! {success: true, payload: servers}),
        StatusCode::OK,
    ))
}

async fn download_jar_and_start(
    version: MinecraftVersion,
    patch: u32,
    kind: ServerKind,
    cfg: ServerConfig,
) -> errors::Result<()> {
    let (mut file, _) = cacher::cache_jar(version, patch, kind.to_string())?;
    let resp = kind
        .download_response(&mut ServerVersion {
            patch: Some(patch),
            minecraft: version,
        })
        .await?;
    tokio::spawn(async move {
        file.write_all(
            &resp
                .bytes()
                .await
                .chain_err(|| "failed to download jar file")?,
        )
        .chain_err(|| "failed to write into jar file")?;
        kind.launch(cfg, Stdio::piped(), Stdio::piped(), Stdio::piped())?;
        Ok(()) as errors::Result<()>
    });
    Ok(())
}

pub async fn start_server(name: String) -> std::result::Result<impl warp::Reply, warp::Rejection> {
    match servers::Server::get(&name) {
        Ok(server) => {
            let kind = server.config.kind.parse::<ServerKind>();
            if let Err(e) = kind {
                return Ok(with_status(
                    json::stringify(json::object! {success: false, payload: e.to_string()}),
                    StatusCode::INTERNAL_SERVER_ERROR,
                ));
            }
            let kind = kind.unwrap();
            let key = CachedJarMetaKey {
                kind: server.config.kind.clone(),
                version: server.config.version.minecraft,
            };
            let cached = cacher::get_cached_patch(&key);
            if let Err(e) = cached {
                return Ok(with_status(
                    json::stringify(json::object! {success: false, payload: e.to_string()}),
                    StatusCode::INTERNAL_SERVER_ERROR,
                ));
            }
            let cached = cached.unwrap();
            match cached {
                Some(patch) => {
                    let latest_patch = route_try!(
                        kind.get_latest_patch(&server.config.version.minecraft)
                            .await
                    );

                    if latest_patch > patch {
                        route_try!(
                            download_jar_and_start(
                                server.config.version.minecraft,
                                latest_patch,
                                kind,
                                server.config
                            )
                            .await
                        );
                        return Ok(with_status(
                            json::stringify(json::object! {success: true, payload: {}}),
                            StatusCode::ACCEPTED,
                        ));
                    } else {
                        route_try!(kind.launch(
                            server.config,
                            Stdio::piped(),
                            Stdio::piped(),
                            Stdio::piped()
                        ));
                    }
                }
                None => {
                    route_try!(
                        download_jar_and_start(
                            server.config.version.minecraft,
                            route_try!(
                                kind.get_latest_patch(&server.config.version.minecraft)
                                    .await
                            ),
                            kind,
                            server.config
                        )
                        .await
                    );
                    return Ok(with_status(
                        json::stringify(json::object! {success: true, payload: {}}),
                        StatusCode::ACCEPTED,
                    ));
                }
            }
            Ok(warp::reply::with_status(
                json::stringify(json::object! {success: true, payload: {}}),
                http::StatusCode::OK,
            ))
        }
        Err(error) => Ok(warp::reply::with_status(
            json::stringify(json::object! {success: false, payload: error.to_string()}),
            http::StatusCode::NOT_FOUND,
        )),
    }
}
