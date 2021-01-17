use std::process::Stdio;

use servers::{iter_servers, CachedJar};
use servo::servers;
use warp::Filter;

mod communication;

#[tokio::main]
async fn main() {
    let server_path = warp::path!("server" / String);
    let get_server = warp::get()
        .and(server_path)
        .map(|name: String| get_server(&name));
    let get_servers = warp::get()
        .and(warp::path!("server"))
        .map(|| list_servers());

    let start_server = warp::path!("server" / String / "start").map(|name: String| {
        match servers::Server::get(&name) {
            Ok(server) => {
                let jar = CachedJar::download(server.config.version);
                if let Err(error) = jar {
                    return json::stringify(
                        json::object! {success: false, payload: error.to_string()},
                    );
                }
                let jar = jar.unwrap();
                let child =
                    jar.start_server(server, Stdio::piped(), Stdio::piped(), Stdio::piped());
                if let Err(error) = child {
                    return json::stringify(
                        json::object! {success: false, payload: error.to_string()},
                    );
                }
                json::stringify(json::object! {success: true, payload: {}})
            }
            Err(error) => {
                json::stringify(json::object! {success: false, payload: error.to_string()})
            }
        }
    });

    warp::serve(get_server.or(get_servers).or(start_server))
        .run(([192, 168, 0, 11], 8080))
        .await;
}

fn get_server(name: &str) -> String {
    let server = servers::Server::get(name);
    match server {
        Ok(server) => json::stringify(json::object! {
            success: true,
            payload: {
                version: format!("{}", server.config.version)
            }
        }),
        Err(error) => json::stringify(json::object! {success: false, payload: error.to_string()}),
    }
}

fn list_servers() -> String {
    let servers_iter = iter_servers();
    if let Err(error) = servers_iter {
        return json::stringify(json::object! {success: false, payload: error.to_string()});
    }
    let mut servers = vec![];
    for i in servers_iter.unwrap() {
        if let Err(error) = i {
            return json::stringify(json::object! {success: false, payload: error.to_string()});
        }
        let server = i.unwrap();
        servers.push(
            json::object! {name: server.config.name, version: format!("{}", server.config.version)},
        );
    }
    json::stringify(json::object! {success: true, payload: servers})
}
