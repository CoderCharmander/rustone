use std::process::Stdio;

use http::StatusCode;
use json::JsonValue;
use rustone::{cacher::CachedJar, servers};

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

pub async fn start_server(name: String) -> std::result::Result<impl warp::Reply, warp::Rejection> {
    match servers::Server::get(&name) {
        Ok(server) => {
            let jar = CachedJar::download(server.config.version).await;
            if let Err(error) = jar {
                return Ok(warp::reply::with_status(
                    json::stringify(json::object! {success: false, payload: error.to_string()}),
                    http::StatusCode::INTERNAL_SERVER_ERROR,
                ));
            }
            let jar = jar.unwrap();
            let child = jar.start_server(server, Stdio::piped(), Stdio::piped(), Stdio::piped());
            if let Err(error) = child {
                return Ok(warp::reply::with_status(
                    json::stringify(json::object! {success: false, payload: error.to_string()}),
                    http::StatusCode::INTERNAL_SERVER_ERROR,
                ));
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
