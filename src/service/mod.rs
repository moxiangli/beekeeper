use std::net::SocketAddr;

use url::Url;

use crate::{
    docker::docker::{Docker, EventsOptions},
    State,
};
use tide::{Request, Response, Result, StatusCode};

use serde::{Deserialize, Serialize};

pub mod container;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlotCount {
    count: i32,
}

pub async fn plot_complete(mut req: Request<State>) -> Result {
    let ip = if let Some(remote) = req.remote() {
        if let Ok(addr) = remote.parse::<SocketAddr>() {
            addr.ip().to_string()
        } else {
            "unknown".to_owned()
        }
    } else {
        "unknown".to_owned()
    };

    let data = req.body_json::<PlotCount>().await?;

    log::info!("get request from {}, body {:?}", ip, data);

    let insert = "insert into plot_complete_info (data_id, from_ip, plot_count, create_time) values (uuid(), ?, ?, sysdate())";
    let state = req.state();

    let ret = sqlx::query(insert)
        .bind(&ip)
        .bind(&data.count)
        .execute(&state.db)
        .await?;
    log::info!("plot complete from {}, {:?}", &ip, ret);

    let response = Response::new(200);

    Ok(response)
}

pub fn docker_not_found_error() -> tide::Error {
    tide::Error::from_str(StatusCode::InternalServerError, "docker not found.")
}

pub async fn docker_info(req: Request<State>) -> Result {
    let url = req.ext::<Url>().ok_or(docker_not_found_error())?;
    let docker = Docker::host(url.clone());
    let response = req.state().send(docker.info()?).await?;
    Ok(tide::Response::from_res(response))
}

pub async fn docker_ping(req: Request<State>) -> Result {
    let url = req.ext::<Url>().ok_or(docker_not_found_error())?;
    let docker = Docker::host(url.clone());
    let response = req.state().send(docker.ping()?).await?;
    Ok(tide::Response::from_res(response))
}

pub async fn docker_events(req: Request<State>) -> Result {
    let url = req.ext::<Url>().ok_or(docker_not_found_error())?;
    let docker = Docker::host(url.clone());
    let options = EventsOptions::builder().build();
    let response = req.state().send(docker.events(&options)?).await?;
    Ok(tide::Response::from_res(response))
}

pub async fn docker_version(req: Request<State>) -> Result {
    let url = req.ext::<Url>().ok_or(docker_not_found_error())?;
    let docker = Docker::host(url.clone());
    let response = req.state().send(docker.version()?).await?;
    Ok(tide::Response::from_res(response))
}
