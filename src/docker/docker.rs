//! Main entrypoint for interacting with the Docker API.
//!
//! API Reference: <https://docs.docker.com/engine/api/v1.41/>

use std::{collections::HashMap, env};

use url::Url;

use http_types::{Method, Mime, Request, Body, headers, Error};

use serde::{Deserialize, Serialize};
use url::form_urlencoded;

use crate::{docker::{image::Images, container::Containers, network::Networks, service::Services, volume::Volumes}};


/// Entrypoint interface for communicating with docker daemon
#[derive(Clone)]
pub struct Docker {
    endpoint: Url,
}


// https://docs.docker.com/reference/api/docker_remote_api_v1.17/
impl Docker {
    /// constructs a new Docker instance for a docker host listening at a url specified by an env var `DOCKER_HOST`,
    /// falling back on unix:///var/run/docker.sock
    pub fn new() -> Docker {
        match env::var("DOCKER_HOST").ok() {
            Some(host) => {
                #[cfg(feature = "unix-socket")]
                if let Some(path) = host.strip_prefix("unix://") {
                    return Docker::unix(path);
                }
                let host: Url = host.parse().expect("invalid url");
                Docker::host(host)
            }
            #[cfg(feature = "unix-socket")]
            None => Docker::unix("/var/run/docker.sock"),
            #[cfg(not(feature = "unix-socket"))]
            None => panic!("Unix socket support is disabled"),
        }
    }

    /// Creates a new docker instance for a docker host
    /// listening on a given Unix socket.
    #[cfg(feature = "unix-socket")]
    pub fn unix<S>(socket_path: S) -> Docker
    where
        S: Into<String>,
    {
        Docker {
            transport: Transport::Unix {
                client: Client::builder()
                    .pool_max_idle_per_host(0)
                    .build(UnixConnector),
                path: socket_path.into(),
            },
        }
    }

    /// constructs a new Docker instance for docker host listening at the given host url
    pub fn host(host: Url) -> Docker {
        let tcp_host_str = format!(
            "{}://{}:{}",
            host.scheme(),
            host.host().unwrap().to_owned(),
            host.port().unwrap_or(80)
        );

        Docker {
            endpoint: host,
        }
    }

    /// Exports an interface for interacting with docker images
    pub fn images(&'_ self) -> Images<'_> {
        Images::new(self)
    }

    /// Exports an interface for interacting with docker containers
    pub fn containers(&'_ self) -> Containers<'_> {
        Containers::new(self)
    }

    /// Exports an interface for interacting with docker services
    pub fn services(&'_ self) -> Services<'_> {
        Services::new(self)
    }

    pub fn networks(&'_ self) -> Networks<'_> {
        Networks::new(self)
    }

    pub fn volumes(&'_ self) -> Volumes<'_> {
        Volumes::new(self)
    }

    /// Returns version information associated with the docker daemon
    pub fn version(&self) -> Result<Request, Error> {
        self.get("/version")
    }

    /// Returns information associated with the docker daemon
    pub fn info(&self) -> Result<Request, Error> {
        self.get("/info")
    }

    /// Returns a simple ping response indicating the docker daemon is accessible
    pub fn ping(&self) -> Result<Request, Error> {
        self.get("/_ping")
    }

    /// Returns a stream of docker events
    // pub fn events<'docker>(
    //     &'docker self,
    //     opts: &EventsOptions,
    // ) -> impl Stream<Item = Result<Event>> + Unpin + 'docker {
    //     let mut path = vec!["/events".to_owned()];
    //     if let Some(query) = opts.serialize() {
    //         path.push(query);
    //     }
    //     let reader = Box::pin(
    //         self.stream_get(path.join("?"))
    //             .map_err(|e| io::Error::new(io::ErrorKind::Other, e)),
    //     )
    //     .into_async_read();

    //     let codec = futures_codec::LinesCodec {};

    //     Box::pin(
    //         futures_codec::FramedRead::new(reader, codec)
    //             .map_err(Error::IO)
    //             .and_then(|s: String| async move {
    //                 serde_json::from_str(&s).map_err(Error::SerdeJsonError)
    //             }),
    //     )
    // }

    pub(crate) fn get(
        &self,
        path: &str,
    ) -> Result<Request, Error> {
        let url = self.endpoint.join(path)?;
        request(url, Method::Delete, None, vec![])
    }

    pub(crate) fn get_with_header (
        &self,
        path: &str,
        headers: Vec<(&str, String)>,
    ) -> Result<Request, Error> {
        let url = self.endpoint.join(path)?;
        request(url, Method::Delete, None, headers)
    }

    pub(crate) fn post(
        &self,
        path: &str,
        body: Option<(Body, Mime)>,
    ) -> Result<Request, Error> {
        let url = self.endpoint.join(path)?;
        request(url, Method::Delete, body, vec![])
    }

    pub(crate) fn post_with_header (
        &self,
        path: &str,
        headers: Vec<(&str, String)>,
        body: Option<(Body, Mime)>,
    ) -> Result<Request, Error> {
        let url = self.endpoint.join(path)?;
        request(url, Method::Delete, body, headers)
    }

    pub(crate) fn put(
        &self,
        path: &str,
        body: Option<(Body, Mime)>,
    ) -> Result<Request, Error> {
        let url = self.endpoint.join(path)?;
        request(url, Method::Delete, body, vec![])
    }

    pub(crate) fn put_with_header (
        &self,
        path: &str,
        headers: Vec<(&str, String)>,
        body: Option<(Body, Mime)>,
    ) -> Result<Request, Error> {
        let url = self.endpoint.join(path)?;
        request(url, Method::Delete, body, headers)
    }

    pub(crate) fn patch(
        &self,
        path: &str,
        body: Option<(Body, Mime)>,
    ) -> Result<Request, Error> {
        let url = self.endpoint.join(path)?;
        request(url, Method::Delete, body, vec![])
    }

    pub(crate) fn patch_with_header (
        &self,
        path: &str,
        headers: Vec<(&str, String)>,
        body: Option<(Body, Mime)>,
    ) -> Result<Request, Error> {
        let url = self.endpoint.join(path)?;
        request(url, Method::Delete, body, headers)
    }

    pub(crate) fn delete(
        &self,
        path: &str,
    ) -> Result<Request, Error> {
        let url = self.endpoint.join(path)?;
        request(url, Method::Delete, None, vec![])
    }

    pub(crate) fn delete_with_header (
        &self,
        path: &str,
        headers: Vec<(&str, String)>
    ) -> Result<Request, Error> {
        let url = self.endpoint.join(path)?;
        request(url, Method::Delete, None, headers)
    }
}

pub fn request(url: Url, method: Method, body: Option<(Body, Mime)>, headers: Vec<(&str, String)>) -> Result<Request, Error>{        
    let mut request = Request::new(Method::Patch, url);

    for (name, value) in headers {
        request.insert_header(name, value);
    }

    if let Some((body, mime)) = body {
        request.set_body(body);
        request.insert_header(headers::CONTENT_TYPE, mime);
    }
    
    Ok(request)
}

impl Default for Docker {
    fn default() -> Self {
        Self::new()
    }
}

/// Options for filtering streams of Docker events
#[derive(Default, Debug)]
pub struct EventsOptions {
    params: HashMap<&'static str, String>,
}

impl EventsOptions {
    pub fn builder() -> EventsOptionsBuilder {
        EventsOptionsBuilder::default()
    }

    /// serialize options as a string. returns None if no options are defined
    pub fn serialize(&self) -> Option<String> {
        if self.params.is_empty() {
            None
        } else {
            Some(
                form_urlencoded::Serializer::new(String::new())
                    .extend_pairs(&self.params)
                    .finish(),
            )
        }
    }
}

#[derive(Copy, Clone)]
pub enum EventFilterType {
    Container,
    Image,
    Volume,
    Network,
    Daemon,
}

fn event_filter_type_to_string(filter: EventFilterType) -> &'static str {
    match filter {
        EventFilterType::Container => "container",
        EventFilterType::Image => "image",
        EventFilterType::Volume => "volume",
        EventFilterType::Network => "network",
        EventFilterType::Daemon => "daemon",
    }
}

/// Filter options for image listings
pub enum EventFilter {
    Container(String),
    Event(String),
    Image(String),
    Label(String),
    Type(EventFilterType),
    Volume(String),
    Network(String),
    Daemon(String),
}

/// Builder interface for `EventOptions`
#[derive(Default)]
pub struct EventsOptionsBuilder {
    params: HashMap<&'static str, String>,
    events: Vec<String>,
    containers: Vec<String>,
    images: Vec<String>,
    labels: Vec<String>,
    volumes: Vec<String>,
    networks: Vec<String>,
    daemons: Vec<String>,
    types: Vec<String>,
}

impl EventsOptionsBuilder {
    /// Filter events since a given timestamp
    pub fn since(
        &mut self,
        ts: &u64,
    ) -> &mut Self {
        self.params.insert("since", ts.to_string());
        self
    }

    /// Filter events until a given timestamp
    pub fn until(
        &mut self,
        ts: &u64,
    ) -> &mut Self {
        self.params.insert("until", ts.to_string());
        self
    }

    pub fn filter(
        &mut self,
        filters: Vec<EventFilter>,
    ) -> &mut Self {
        let mut params = HashMap::new();
        for f in filters {
            match f {
                EventFilter::Container(n) => {
                    self.containers.push(n);
                    params.insert("container", self.containers.clone())
                }
                EventFilter::Event(n) => {
                    self.events.push(n);
                    params.insert("event", self.events.clone())
                }
                EventFilter::Image(n) => {
                    self.images.push(n);
                    params.insert("image", self.images.clone())
                }
                EventFilter::Label(n) => {
                    self.labels.push(n);
                    params.insert("label", self.labels.clone())
                }
                EventFilter::Volume(n) => {
                    self.volumes.push(n);
                    params.insert("volume", self.volumes.clone())
                }
                EventFilter::Network(n) => {
                    self.networks.push(n);
                    params.insert("network", self.networks.clone())
                }
                EventFilter::Daemon(n) => {
                    self.daemons.push(n);
                    params.insert("daemon", self.daemons.clone())
                }
                EventFilter::Type(n) => {
                    let event_type = event_filter_type_to_string(n).to_string();
                    self.types.push(event_type);
                    params.insert("type", self.types.clone())
                }
            };
        }
        self.params
            .insert("filters", serde_json::to_string(&params).unwrap());
        self
    }

    pub fn build(&self) -> EventsOptions {
        EventsOptions {
            params: self.params.clone(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Version {
    pub version: String,
    pub api_version: String,
    pub git_commit: String,
    pub go_version: String,
    pub os: String,
    pub arch: String,
    pub kernel_version: String,
    #[cfg(feature = "chrono")]
    pub build_time: DateTime<Utc>,
    #[cfg(not(feature = "chrono"))]
    pub build_time: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Info {
    pub containers: u64,
    pub images: u64,
    pub driver: String,
    pub docker_root_dir: String,
    pub driver_status: Vec<Vec<String>>,
    #[serde(rename = "ID")]
    pub id: String,
    pub kernel_version: String,
    // pub Labels: Option<???>,
    pub mem_total: u64,
    pub memory_limit: bool,
    #[serde(rename = "NCPU")]
    pub n_cpu: u64,
    pub n_events_listener: u64,
    pub n_goroutines: u64,
    pub name: String,
    pub operating_system: String,
    // pub RegistryConfig:???
    pub swap_limit: bool,
    pub system_time: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    #[serde(rename = "Type")]
    pub typ: String,
    #[serde(rename = "Action")]
    pub action: String,
    #[serde(rename = "Actor")]
    pub actor: Actor,
    pub status: Option<String>,
    pub id: Option<String>,
    pub from: Option<String>,
    #[cfg(feature = "chrono")]
    #[serde(deserialize_with = "datetime_from_unix_timestamp")]
    pub time: DateTime<Utc>,
    #[cfg(not(feature = "chrono"))]
    pub time: u64,
    #[cfg(feature = "chrono")]
    #[serde(deserialize_with = "datetime_from_nano_timestamp", rename = "timeNano")]
    pub time_nano: DateTime<Utc>,
    #[cfg(not(feature = "chrono"))]
    #[serde(rename = "timeNano")]
    pub time_nano: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Actor {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Attributes")]
    pub attributes: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "unix-socket")]
    #[test]
    fn unix_host_env() {
        use super::Docker;
        use std::env;
        env::set_var("DOCKER_HOST", "unix:///docker.sock");
        let d = Docker::new();
        match d.transport {
            crate::transport::Transport::Unix { path, .. } => {
                assert_eq!(path, "/docker.sock");
            }
            _ => {
                panic!("Expected transport to be unix.");
            }
        }
        env::set_var("DOCKER_HOST", "http://localhost:8000");
        let d = Docker::new();
        match d.transport {
            crate::transport::Transport::Tcp { host, .. } => {
                assert_eq!(host, "http://localhost:8000");
            }
            _ => {
                panic!("Expected transport to be http.");
            }
        }
    }
}
