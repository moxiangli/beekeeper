use std::net::SocketAddr;

use url::Url;

use crate::{State, docker::docker::Docker};
use tide::{Request, Response, Result, StatusCode};

use serde::{Deserialize, Serialize};

use http_client::HttpClient;

// use shiplift::Docker;

// use bollard::Docker;
// use bollard::API_DEFAULT_VERSION;




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

pub async fn docker_info_shiplift(req: Request<State>) -> Result {
    let url = Url::parse("http://localhost:8010/")?;
    let docker = Docker::host(url);
    let state = req.state();
    log::info!("request for docker info.");
    match docker.info() {
        Ok(request) => {
            log::info!("request for docker {:?}", request);
            let response = state.client.send(request).await?;
            log::info!("response from docker {:?}", response);
            Ok(tide::Response::from_res(response))
        },
        Err(err) => {
            Err(tide::Error::new(StatusCode::InternalServerError, err.into_inner()))
        }
    }
}

// pub async fn docker_info_bollard(_: Request<State>) -> Result {
//     let docker = bollard::Docker::connect_with_http("http://localhost:8010", 30, API_DEFAULT_VERSION);
//     // let docker = Docker::host(Uri::from_static("http://localhost:8010"));

//     match docker {
//         Ok(d) => {
//             let info = d.info().await;
//             match info {
//                 Ok(info) => {
//                     println!("info {:?}", info);
//                     let data = serde_json::to_string(&info)?;
//                     let mut response = Response::new(200);
//                     response.set_body(data);
//                     Ok(response)
//                 },
//                 Err(err) => {
//                     eprintln!("Error: {}", err);
//                     let err = tide::Error::new(StatusCode::InternalServerError, err);
//                     Err(err)
//                 },
//             }
//         },
//         Err(err) => {
//             eprintln!("Error: {}", err);
//             let err = tide::Error::new(StatusCode::InternalServerError, err);
//             Err(err)
//         },
//     }
// }