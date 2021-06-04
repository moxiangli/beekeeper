
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tide::{
    Next,
    Result,
    Server,
    Request,
    Response,
    StatusCode,
};

use sqlx::{MySqlPool, FromRow};
// use serde::{Serialize, Deserialize};

use http_types::{Url, headers::HeaderValue};
use http_client::HttpClient;

use tide::security::{CorsMiddleware, Origin};


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

impl State {
    pub async fn send(&self, request: http_types::Request) -> std::result::Result<http_types::Response, http_types::Error> {
        log::debug!("request to docker: {:?}", request);
        let response = self.client.send(request).await;
        log::debug!("response from docker: {:?}", response);
        response
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct DockerDaemonInfo {
    pub host_ip: String,
    pub docker_port: i32,
}

fn docker_id<'a>(
    mut request: Request<State>,
    next: Next<'a, State>,
) -> Pin<Box<dyn Future<Output = Result> + Send + 'a>> {
    Box::pin(async {
        let _ = &request.state().db;
        let id = request.param("docker");
        if let Ok(id) = id {
            // let sql = "select host_ip, docker_port from host_docker_info where host_id = ?";
            // let docker = sqlx::query_as::<_, DockerDaemonInfo>(sql)
            // .bind(&id)
            // .fetch_one(db)
            // .await?;
            log::debug!("request: {}", request.url());
            log::debug!("request docker: {}", id);

            // let url = String::from("http://127.0.0.1:8010");
            let url = Url::parse("http://127.0.0.1:8010")?;
            request.set_ext(url);
            Ok(next.run(request).await)
        } else {
            Ok(Response::new(StatusCode::BadRequest))
        }
    })
}


#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    logger::logger_init();

    let handle = tokio::runtime::Handle::current();
    let _ = handle.enter();
    

    let pool = MySqlPool::connect("mysql://loop:123456@127.0.0.1:3306/chia").await?;

    let state = State {
        db: pool,
        client: Arc::new(Client::new()),
    };

    let mut app = Server::with_state(state.clone());

    app.at("/chia/plots").post(service::plot_complete);
    app.at("/docker/:docker").nest({
        let mut docker = Server::with_state(state.clone());
        docker.with(docker_id);
        docker.at("info").get(service::docker_info);
        docker.at("ping").get(service::docker_ping);
        docker.at("events").get(service::docker_events);
        docker.at("version").get(service::docker_version);
        
        
        docker.at("containers")
        .get(service::container::list)
        .post(service::container::create);
        docker.at("containers/:id")
        .get(service::container::inspect)
        .nest({
            let mut container = Server::with_state(state.clone());
            container.at("top").get(service::container::top);
            container.at("logs").get(service::container::logs);
            container.at("changes").get(service::container::changes);
            container.at("export").get(service::container::export);
            container.at("stats").get(service::container::stats);
            container.at("start").post(service::container::start);
            container.at("stop").post(service::container::stop);
            container.at("restart").post(service::container::restart);
            container.at("kill").post(service::container::kill);
            container.at("rename").post(service::container::rename);
            container.at("pause").post(service::container::pause);
            container.at("unpause").post(service::container::unpause);
            container.at("attach").post(service::container::attach);
            container.at("wait").post(service::container::wait);
            container.at("remove").post(service::container::remove);
    
            container
        });

        docker
    });

    let rules = CorsMiddleware::new()
    .allow_methods("GET, POST, PUT, DELETE, OPTIONS".parse::<HeaderValue>().unwrap())
    .allow_origin(Origin::from("*"))
    .allow_credentials(false);
    app.with(rules);

    app.listen("127.0.0.1:8030").await?;
    Ok(())
}