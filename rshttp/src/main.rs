use warp::Filter;

//mod communication;
mod routes;

#[tokio::main]
async fn main() {
    let get_server = warp::get()
        .and(warp::path("server"))
        .and(warp::path::param())
        .and(warp::path::end())
        .and_then(routes::get_server);
    let get_servers = warp::get()
        .and(warp::path("server"))
        .and(warp::path::end())
        .and_then(routes::list_servers);

    // /server/<name>/start: start a server
    // returns {success: true} if success, or {success: false, payload: <error string>}
    let start_server_path = warp::get()
        .and(warp::path("server"))
        .and(warp::path::param())
        .and(warp::path("start"))
        .and(warp::path::end())
        .and_then(routes::start_server);

    println!("Serving on 0.0.0.0:8080");

    warp::serve(get_server.or(get_servers).or(start_server_path))
        .run("0.0.0.0:8081".parse::<std::net::SocketAddrV4>().unwrap())
        .await;
}
