use std::sync::Arc;
use tide::Server;

use sqlx::MySqlPool;

// use std::net::ToSocketAddrs;
// use tide::listener::ToListener;
// use tokio::runtime::{Builder, Runtime};

// use shiplift::Docker;
// use bollard::Docker;
// use bollard::API_DEFAULT_VERSION;

// use hyper::Uri;


#[cfg(any(feature = "runtime-std", feature = "docs"))]
use http_client::h1::H1Client as Client;
#[cfg(all(feature = "hyper_client", not(feature = "docs")))]
use http_client::hyper::HyperClient as Client;
#[cfg(all(feature = "curl_client", not(feature = "docs")))]
use http_client::isahc::IsahcClient as Client;
#[cfg(all(feature = "wasm_client", not(feature = "docs")))]
use http_client::wasm::WasmClient as Client;


mod logger;
mod errors;
mod docker;
mod service;


#[derive(Debug, Clone)]
pub struct State {
    pub db: MySqlPool,
    pub client: Arc<Client>,
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    logger::logger_init();

    let handle = tokio::runtime::Handle::current();
    let _ = handle.enter();
    

    let pool = MySqlPool::connect("mysql://loop:123456@127.0.0.1:3306/chia").await?;

    let state = State {
        db: pool,
        client: Arc::new(Client::new()),
    };

    let mut app = Server::with_state(state);
    // app.at("/chia/environment").post(ep);

    // app.at("/chia/status")
    // .post(service::upload_status)
    // .get(ep);

    app.at("/chia/plots").post(service::plot_complete);
    app.at("/docker/info").get(service::docker_info_shiplift);

    // app.listen("172.16.10.10:8030".to_socket_addrs()?.collect::<Vec<_>>()).await?;
    app.listen("127.0.0.1:8030").await?;

    // let listener = TcpListener::<State>::from_addr();
    Ok(())
}

// async fn run() -> Result<(), Box<dyn std::error::Error>> {
// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     logger::logger_init();

//     let runtime: Runtime = Builder::new_current_thread()
//     .worker_threads(2)
//     .enable_all()
//     .build()?;

//     // let pool = MySqlPool::connect("mysql://loop:loop123@172.16.10.30:3306/chia").await?;
//     let connect = MySqlPool::connect("mysql://loop:123456@127.0.0.1:3306/chia");
//     let pool = runtime.block_on(connect)?;

//     let state = State {
//         db: pool,
//     };

//     let mut app = Server::with_state(state);
//     // app.at("/chia/environment").post(ep);

//     // app.at("/chia/status")
//     // .post(service::upload_status)
//     // .get(ep);

//     app.at("/chia/plots").post(service::plot_complete);
//     app.at("/docker/info").get(service::docker_info);

//     // app.listen("172.16.10.10:8030").await?;
//     let f = app.listen("127.0.0.1:8030");
//     runtime.block_on(f)?;
//     Ok(())
// }

// #[tokio::main]
// async fn main() {
//     let docker = Docker::host(Uri::from_static("http://localhost:8010"));

//     match docker.info().await {
//         Ok(info) => println!("info {:?}", info),
//         Err(e) => eprintln!("Error: {}", e),
//     }
// }

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let docker = Docker::connect_with_http("http://localhost:8010", 30, API_DEFAULT_VERSION);
//     // let docker = Docker::host(Uri::from_static("http://localhost:8010"));

//     match docker {
//         Ok(d) => {
//             let info = d.info().await;
//             match info {
//                 Ok(inf) => {
//                     eprintln!("{:?}", inf);
//                 },
//                 Err(err) => eprintln!("Error: {:?}", err),
//             }
//         },
//         Err(e) => eprintln!("Error: {}", e),
//     }

//     Ok(())
// }