use std::default::Default;
use std::time::Duration;
use url::Url;

use crate::docker::container::LogsOptions;
use crate::docker::container::RmContainerOptions;
use crate::docker::{container::ContainerOptions, docker::Docker};
use crate::service::docker_not_found_error;
use crate::State;

use tide::{Request, Result};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct ContainerProcessOptions {
    pub ps_args: Option<String>,
}

#[derive(Deserialize)]
pub struct ContainerLogsOptions {
    pub follow: Option<bool>,
    pub stdout: Option<bool>,
    pub stderr: Option<bool>,
    pub since: Option<i64>,
    pub until: Option<i64>,
    pub timestamps: Option<bool>,
    pub tail: Option<String>,
}

impl Into<LogsOptions> for ContainerLogsOptions {
    fn into(self) -> LogsOptions {
        let mut builder = LogsOptions::builder();
        if let Some(b) = self.follow {
            builder.follow(b);
        }
        if let Some(b) = self.stdout {
            builder.stdout(b);
        }
        if let Some(b) = self.stderr {
            builder.stderr(b);
        }
        if let Some(b) = self.since {
            builder.since(b);
        }
        if let Some(b) = self.timestamps {
            builder.timestamps(b);
        }
        if let Some(b) = self.tail {
            builder.tail(b.as_str());
        }
        builder.build()
    }
}

#[derive(Deserialize)]
pub struct ContainerStopOptions {
    pub wait: Option<u64>,
}

#[derive(Deserialize)]
pub struct ContainerKillOptions {
    pub singal: Option<String>,
}

#[derive(Deserialize)]
pub struct ContainerRenameOptions {
    pub name: String,
}

#[derive(Deserialize)]
pub struct ContainerRemoveOptions {
    pub v: Option<bool>,
    pub force: Option<bool>,
    pub link: Option<bool>,
}

impl Into<RmContainerOptions> for ContainerRemoveOptions {
    fn into(self) -> RmContainerOptions {
        let mut builder = RmContainerOptions::builder();
        if let Some(v) = self.v {
            builder.volumes(v);
        }
        if let Some(f) = self.force {
            builder.force(f);
        }
        builder.build()
    }
}

pub async fn list(req: Request<State>) -> Result {
    let url = req.ext::<Url>().ok_or(docker_not_found_error())?;
    let docker = Docker::host(url.clone());
    let response = req
        .state()
        .send(docker.containers().list(&Default::default())?)
        .await?;
    Ok(tide::Response::from_res(response))
}

pub async fn create(mut req: Request<State>) -> Result {
    let image: ContainerOptions = req.body_json().await?;
    let url = req.ext::<Url>().ok_or(docker_not_found_error())?;

    let docker = Docker::host(url.clone());
    let response = req
        .state()
        .send(docker.containers().create(&image)?)
        .await?;
    Ok(tide::Response::from_res(response))
}

pub async fn inspect(req: Request<State>) -> Result {
    let id = req.param("id")?;
    let url = req.ext::<Url>().ok_or(docker_not_found_error())?;
    let docker = Docker::host(url.clone());
    let response = req
        .state()
        .send(docker.containers().get(id).inspect()?)
        .await?;
    Ok(tide::Response::from_res(response))
}

pub async fn top(req: Request<State>) -> Result {
    let id = req.param("id")?;
    let args = req.query::<ContainerProcessOptions>()?;
    let url = req.ext::<Url>().ok_or(docker_not_found_error())?;
    let docker = Docker::host(url.clone());
    let response = req
        .state()
        .send(docker.containers().get(id).top(args.ps_args)?)
        .await?;
    Ok(tide::Response::from_res(response))
}

pub async fn logs(req: Request<State>) -> Result {
    let id = req.param("id")?;
    let args = req.query::<ContainerLogsOptions>()?;
    let url = req.ext::<Url>().ok_or(docker_not_found_error())?;
    let docker = Docker::host(url.clone());
    let response = req
        .state()
        .send(docker.containers().get(id).logs(&args.into())?)
        .await?;
    Ok(tide::Response::from_res(response))
}

pub async fn changes(req: Request<State>) -> Result {
    let id = req.param("id")?;
    let url = req.ext::<Url>().ok_or(docker_not_found_error())?;
    let docker = Docker::host(url.clone());
    let response = req
        .state()
        .send(docker.containers().get(id).changes()?)
        .await?;
    Ok(tide::Response::from_res(response))
}

pub async fn export(req: Request<State>) -> Result {
    let id = req.param("id")?;
    let url = req.ext::<Url>().ok_or(docker_not_found_error())?;
    let docker = Docker::host(url.clone());
    let response = req
        .state()
        .send(docker.containers().get(id).export()?)
        .await?;
    Ok(tide::Response::from_res(response))
}

pub async fn stats(req: Request<State>) -> Result {
    let id = req.param("id")?;
    let url = req.ext::<Url>().ok_or(docker_not_found_error())?;
    let docker = Docker::host(url.clone());
    let response = req
        .state()
        .send(docker.containers().get(id).stats()?)
        .await?;
    Ok(tide::Response::from_res(response))
}

// resize not impl

pub async fn start(req: Request<State>) -> Result {
    let id = req.param("id")?;
    let url = req.ext::<Url>().ok_or(docker_not_found_error())?;
    let docker = Docker::host(url.clone());
    let response = req
        .state()
        .send(docker.containers().get(id).start()?)
        .await?;
    Ok(tide::Response::from_res(response))
}

pub async fn stop(req: Request<State>) -> Result {
    let id = req.param("id")?;
    let time = req.query::<ContainerStopOptions>()?;
    let time = if let Some(t) = time.wait {
        Some(Duration::from_secs(t))
    } else {
        None
    };
    let url = req.ext::<Url>().ok_or(docker_not_found_error())?;
    let docker = Docker::host(url.clone());
    let response = req
        .state()
        .send(docker.containers().get(id).stop(time)?)
        .await?;
    Ok(tide::Response::from_res(response))
}

pub async fn restart(req: Request<State>) -> Result {
    let id = req.param("id")?;
    let time = req.query::<ContainerStopOptions>()?;
    let time = if let Some(t) = time.wait {
        Some(Duration::from_secs(t))
    } else {
        None
    };
    let url = req.ext::<Url>().ok_or(docker_not_found_error())?;
    let docker = Docker::host(url.clone());
    let response = req
        .state()
        .send(docker.containers().get(id).restart(time)?)
        .await?;
    Ok(tide::Response::from_res(response))
}

pub async fn kill(req: Request<State>) -> Result {
    let id = req.param("id")?;
    let options = req.query::<ContainerKillOptions>()?;
    let url = req.ext::<Url>().ok_or(docker_not_found_error())?;
    let docker = Docker::host(url.clone());
    let response = req
        .state()
        .send(docker.containers().get(id).kill(options.singal)?)
        .await?;
    Ok(tide::Response::from_res(response))
}

pub async fn rename(req: Request<State>) -> Result {
    let id = req.param("id")?;
    let options = req.query::<ContainerRenameOptions>()?;
    let url = req.ext::<Url>().ok_or(docker_not_found_error())?;
    let docker = Docker::host(url.clone());
    let response = req
        .state()
        .send(docker.containers().get(id).rename(options.name.as_str())?)
        .await?;
    Ok(tide::Response::from_res(response))
}

pub async fn pause(req: Request<State>) -> Result {
    let id = req.param("id")?;
    let url = req.ext::<Url>().ok_or(docker_not_found_error())?;
    let docker = Docker::host(url.clone());
    let response = req
        .state()
        .send(docker.containers().get(id).pause()?)
        .await?;
    Ok(tide::Response::from_res(response))
}

pub async fn unpause(req: Request<State>) -> Result {
    let id = req.param("id")?;
    let url = req.ext::<Url>().ok_or(docker_not_found_error())?;
    let docker = Docker::host(url.clone());
    let response = req
        .state()
        .send(docker.containers().get(id).unpause()?)
        .await?;
    Ok(tide::Response::from_res(response))
}

pub async fn attach(req: Request<State>) -> Result {
    let id = req.param("id")?;
    let url = req.ext::<Url>().ok_or(docker_not_found_error())?;
    let docker = Docker::host(url.clone());
    let response = req
        .state()
        .send(docker.containers().get(id).attach()?)
        .await?;
    Ok(tide::Response::from_res(response))
}

pub async fn wait(req: Request<State>) -> Result {
    let id = req.param("id")?;
    let url = req.ext::<Url>().ok_or(docker_not_found_error())?;
    let docker = Docker::host(url.clone());
    let response = req
        .state()
        .send(docker.containers().get(id).wait()?)
        .await?;
    Ok(tide::Response::from_res(response))
}

pub async fn remove(req: Request<State>) -> Result {
    let id = req.param("id")?;
    let options = req.query::<ContainerRemoveOptions>()?;
    let url = req.ext::<Url>().ok_or(docker_not_found_error())?;
    let docker = Docker::host(url.clone());
    let response = req
        .state()
        .send(docker.containers().get(id).remove(options.into())?)
        .await?;
    Ok(tide::Response::from_res(response))
}
