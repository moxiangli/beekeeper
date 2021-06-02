//! Create and manage images.
//!
//! API Reference: <https://docs.docker.com/engine/api/v1.41/#tag/Image>

use std::{collections::HashMap, io::Read, iter};

use serde::{Deserialize, Serialize};
use url::form_urlencoded;

use http_types::{Request, Body, Error};
use crate::docker::{docker::Docker, tarball, tar};

/// Interface for accessing and manipulating a named docker image
///
/// Api Reference: <https://docs.docker.com/engine/api/v1.41/#tag/Image>
pub struct Image<'docker> {
    docker: &'docker Docker,
    name: String,
}

impl<'docker> Image<'docker> {
    /// Exports an interface for operations that may be performed against a named image
    pub fn new<S>(
        docker: &'docker Docker,
        name: S,
    ) -> Self
    where
        S: Into<String>,
    {
        Image {
            docker,
            name: name.into(),
        }
    }

    pub  fn inspect(&self) -> Result<Request, Error> {
        self.docker.get(&format!("/images/{}/json", self.name))
    }

    pub  fn history(&self) -> Result<Request, Error> {
        self.docker.get(&format!("/images/{}/history", self.name))
    }

    pub  fn delete(&self) -> Result<Request, Error> {
        self.docker.delete(&format!("/images/{}", self.name))
    }

    /// Export this image to a tarball
    ///
    /// Api Reference: <https://docs.docker.com/engine/api/v1.41/#operation/ImageGet>
    pub fn export(&self) -> Result<Request, Error> {
        self.docker.get(&format!("/images/{}/get", self.name))
    }

    /// Adds a tag to an image
    ///
    /// Api Reference: <https://docs.docker.com/engine/api/v1.41/#operation/ImageTag>
    pub  fn tag(
        &self,
        opts: &TagOptions,
    ) -> Result<Request, Error> {
        let mut path = vec![format!("/images/{}/tag", self.name)];
        if let Some(query) = opts.serialize() {
            path.push(query)
        }
        self.docker.post(&path.join("?"),  None)
    }
}

/// Interface for docker images
pub struct Images<'docker> {
    docker: &'docker Docker,
}

impl<'docker> Images<'docker> {
    /// Exports an interface for interacting with docker images
    pub fn new(docker: &'docker Docker) -> Self {
        Images { docker }
    }

    /// Builds a new image build by reading a Dockerfile in a target directory
    ///
    /// Api Reference: <https://docs.docker.com/engine/api/v1.41/#operation/ImageBuild>
    pub fn build(
        &self,
        opts: &BuildOptions,
    ) -> Result<Request, Error> {
        let mut path = vec!["/build".to_owned()];
        if let Some(query) = opts.serialize() {
            path.push(query)
        }

        // To not tie the lifetime of `opts` to the 'stream, we do the tarring work outside of the
        // stream. But for backwards compatability, we have to return the error inside of the
        // stream.
        let mut bytes = Vec::default();
        let _ = tarball::dir(&mut bytes, opts.path.as_str())?;

        // We must take ownership of the Docker reference. If we don't then the lifetime of 'stream
        // is incorrectly tied to `self`.
        self.docker.post(&path.join("?"),  Some((Body::from(bytes), tar())))
    }

    /// Lists the docker images on the current docker host
    ///
    /// Api Reference: <https://docs.docker.com/engine/api/v1.41/#operation/ImageList>
    pub  fn list(
        &self,
        opts: &ImageListOptions,
    ) -> Result<Request, Error> {
        let mut path = vec!["/images/json".to_owned()];
        if let Some(query) = opts.serialize() {
            path.push(query);
        }
        self.docker.get(&path.join("?"))
    }

    /// Returns a reference to a set of operations available for a named image
    pub fn get<S>(
        &self,
        name: S,
    ) -> Image<'docker>
    where
        S: Into<String>,
    {
        Image::new(self.docker, name)
    }

    /// Search for docker images by term
    ///
    /// Api Reference: <https://docs.docker.com/engine/api/v1.41/#operation/ImageSearch>
    pub fn search(
        &self,
        term: &str,
    ) -> Result<Request, Error> {
        let query = form_urlencoded::Serializer::new(String::new())
            .append_pair("term", term)
            .finish();
        self.docker.get(&format!("/images/search?{}", query))
    }

    /// Pull and create a new docker images from an existing image
    ///
    /// Api Reference: <https://docs.docker.com/engine/api/v1.41/#operation/ImagePull>
    pub fn pull(
        &self,
        opts: &PullOptions,
    ) -> Result<Request, Error> {
        let mut path = vec!["/images/create".to_owned()];
        if let Some(query) = opts.serialize() {
            path.push(query);
        }
        let headers = opts
            .auth_header()
            .map(|a| iter::once(("X-Registry-Auth", a)));
        self.docker.post(&path.join("?"), None)
    }

    /// exports a collection of named images,
    /// either by name, name:tag, or image id, into a tarball
    ///
    /// Api Reference: <https://docs.docker.com/engine/api/v1.41/#operation/ImageGetAll>
    pub fn export(
        &self,
        names: Vec<&str>,
    ) -> Result<Request, Error> {
        let params = names.iter().map(|n| ("names", *n));
        let query = form_urlencoded::Serializer::new(String::new())
            .extend_pairs(params)
            .finish();
        self.docker.get(&format!("/images/get?{}", query))
    }

    /// imports an image or set of images from a given tarball source
    /// source can be uncompressed on compressed via gzip, bzip2 or xz
    ///
    /// Api Reference: <https://docs.docker.com/engine/api/v1.41/#operation/ImageLoad>
    pub fn import<R>(
        self,
        mut tarball: R,
    ) -> Result<Request, Error>
    where
        R: Read + Send + 'docker,
    {
        let mut bytes = Vec::default();

        tarball.read_to_end(&mut bytes)?;

        self.docker.post("/images/load", Some((Body::from(bytes), tar())))
    }
}

#[derive(Clone, Serialize, Debug)]
#[serde(untagged)]
pub enum RegistryAuth {
    Password {
        username: String,
        password: String,

        #[serde(skip_serializing_if = "Option::is_none")]
        email: Option<String>,

        #[serde(rename = "serveraddress")]
        #[serde(skip_serializing_if = "Option::is_none")]
        server_address: Option<String>,
    },
    Token {
        #[serde(rename = "identitytoken")]
        identity_token: String,
    },
}

impl RegistryAuth {
    /// return a new instance with token authentication
    pub fn token<S>(token: S) -> RegistryAuth
    where
        S: Into<String>,
    {
        RegistryAuth::Token {
            identity_token: token.into(),
        }
    }

    /// return a new instance of a builder for authentication
    pub fn builder() -> RegistryAuthBuilder {
        RegistryAuthBuilder::default()
    }

    /// serialize authentication as JSON in base64
    pub fn serialize(&self) -> String {
        serde_json::to_string(self)
            .map(|c| base64::encode_config(&c, base64::URL_SAFE))
            .unwrap()
    }
}

#[derive(Default)]
pub struct RegistryAuthBuilder {
    username: Option<String>,
    password: Option<String>,
    email: Option<String>,
    server_address: Option<String>,
}

impl RegistryAuthBuilder {
    pub fn username<I>(
        &mut self,
        username: I,
    ) -> &mut Self
    where
        I: Into<String>,
    {
        self.username = Some(username.into());
        self
    }

    pub fn password<I>(
        &mut self,
        password: I,
    ) -> &mut Self
    where
        I: Into<String>,
    {
        self.password = Some(password.into());
        self
    }

    pub fn email<I>(
        &mut self,
        email: I,
    ) -> &mut Self
    where
        I: Into<String>,
    {
        self.email = Some(email.into());
        self
    }

    pub fn server_address<I>(
        &mut self,
        server_address: I,
    ) -> &mut Self
    where
        I: Into<String>,
    {
        self.server_address = Some(server_address.into());
        self
    }

    pub fn build(&self) -> RegistryAuth {
        RegistryAuth::Password {
            username: self.username.clone().unwrap_or_else(String::new),
            password: self.password.clone().unwrap_or_else(String::new),
            email: self.email.clone(),
            server_address: self.server_address.clone(),
        }
    }
}

#[derive(Default, Debug)]
pub struct TagOptions {
    pub params: HashMap<&'static str, String>,
}

impl TagOptions {
    /// return a new instance of a builder for options
    pub fn builder() -> TagOptionsBuilder {
        TagOptionsBuilder::default()
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

#[derive(Default)]
pub struct TagOptionsBuilder {
    params: HashMap<&'static str, String>,
}

impl TagOptionsBuilder {
    pub fn repo<R>(
        &mut self,
        r: R,
    ) -> &mut Self
    where
        R: Into<String>,
    {
        self.params.insert("repo", r.into());
        self
    }

    pub fn tag<T>(
        &mut self,
        t: T,
    ) -> &mut Self
    where
        T: Into<String>,
    {
        self.params.insert("tag", t.into());
        self
    }

    pub fn build(&self) -> TagOptions {
        TagOptions {
            params: self.params.clone(),
        }
    }
}

#[derive(Default, Debug)]
pub struct PullOptions {
    auth: Option<RegistryAuth>,
    params: HashMap<&'static str, String>,
}

impl PullOptions {
    /// return a new instance of a builder for options
    pub fn builder() -> PullOptionsBuilder {
        PullOptionsBuilder::default()
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

    pub(crate) fn auth_header(&self) -> Option<String> {
        self.auth.clone().map(|a| a.serialize())
    }
}

#[derive(Default)]
pub struct PullOptionsBuilder {
    auth: Option<RegistryAuth>,
    params: HashMap<&'static str, String>,
}

impl PullOptionsBuilder {
    ///  Name of the image to pull. The name may include a tag or digest.
    /// This parameter may only be used when pulling an image.
    /// If an untagged value is provided and no `tag` is provided, _all_
    /// tags will be pulled
    /// The pull is cancelled if the HTTP connection is closed.
    pub fn image<I>(
        &mut self,
        img: I,
    ) -> &mut Self
    where
        I: Into<String>,
    {
        self.params.insert("fromImage", img.into());
        self
    }

    pub fn src<S>(
        &mut self,
        s: S,
    ) -> &mut Self
    where
        S: Into<String>,
    {
        self.params.insert("fromSrc", s.into());
        self
    }

    /// Repository name given to an image when it is imported. The repo may include a tag.
    /// This parameter may only be used when importing an image.
    pub fn repo<R>(
        &mut self,
        r: R,
    ) -> &mut Self
    where
        R: Into<String>,
    {
        self.params.insert("repo", r.into());
        self
    }

    /// Tag or digest. If empty when pulling an image,
    /// this causes all tags for the given image to be pulled.
    pub fn tag<T>(
        &mut self,
        t: T,
    ) -> &mut Self
    where
        T: Into<String>,
    {
        self.params.insert("tag", t.into());
        self
    }

    pub fn auth(
        &mut self,
        auth: RegistryAuth,
    ) -> &mut Self {
        self.auth = Some(auth);
        self
    }

    pub fn build(&mut self) -> PullOptions {
        PullOptions {
            auth: self.auth.take(),
            params: self.params.clone(),
        }
    }
}

#[derive(Default, Debug)]
pub struct BuildOptions {
    pub path: String,
    params: HashMap<&'static str, String>,
}

impl BuildOptions {
    /// return a new instance of a builder for options
    /// path is expected to be a file path to a directory containing a Dockerfile
    /// describing how to build a Docker image
    pub fn builder<S>(path: S) -> BuildOptionsBuilder
    where
        S: Into<String>,
    {
        BuildOptionsBuilder::new(path)
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

#[derive(Default)]
pub struct BuildOptionsBuilder {
    path: String,
    params: HashMap<&'static str, String>,
}

impl BuildOptionsBuilder {
    /// path is expected to be a file path to a directory containing a Dockerfile
    /// describing how to build a Docker image
    pub(crate) fn new<S>(path: S) -> Self
    where
        S: Into<String>,
    {
        BuildOptionsBuilder {
            path: path.into(),
            ..Default::default()
        }
    }

    /// set the name of the docker file. defaults to "DockerFile"
    pub fn dockerfile<P>(
        &mut self,
        path: P,
    ) -> &mut Self
    where
        P: Into<String>,
    {
        self.params.insert("dockerfile", path.into());
        self
    }

    /// tag this image with a name after building it
    pub fn tag<T>(
        &mut self,
        t: T,
    ) -> &mut Self
    where
        T: Into<String>,
    {
        self.params.insert("t", t.into());
        self
    }

    pub fn remote<R>(
        &mut self,
        r: R,
    ) -> &mut Self
    where
        R: Into<String>,
    {
        self.params.insert("remote", r.into());
        self
    }

    /// don't use the image cache when building image
    pub fn nocache(
        &mut self,
        nc: bool,
    ) -> &mut Self {
        self.params.insert("nocache", nc.to_string());
        self
    }

    pub fn rm(
        &mut self,
        r: bool,
    ) -> &mut Self {
        self.params.insert("rm", r.to_string());
        self
    }

    pub fn forcerm(
        &mut self,
        fr: bool,
    ) -> &mut Self {
        self.params.insert("forcerm", fr.to_string());
        self
    }

    /// `bridge`, `host`, `none`, `container:<name|id>`, or a custom network name.
    pub fn network_mode<T>(
        &mut self,
        t: T,
    ) -> &mut Self
    where
        T: Into<String>,
    {
        self.params.insert("networkmode", t.into());
        self
    }

    pub fn memory(
        &mut self,
        memory: u64,
    ) -> &mut Self {
        self.params.insert("memory", memory.to_string());
        self
    }

    pub fn cpu_shares(
        &mut self,
        cpu_shares: u32,
    ) -> &mut Self {
        self.params.insert("cpushares", cpu_shares.to_string());
        self
    }

    // todo: memswap
    // todo: cpusetcpus
    // todo: cpuperiod
    // todo: cpuquota
    // todo: buildargs

    pub fn build(&self) -> BuildOptions {
        BuildOptions {
            path: self.path.clone(),
            params: self.params.clone(),
        }
    }
}

/// Filter options for image listings
pub enum ImageFilter {
    Dangling,
    LabelName(String),
    Label(String, String),
}

/// Options for filtering image list results
#[derive(Default, Debug)]
pub struct ImageListOptions {
    params: HashMap<&'static str, String>,
}

impl ImageListOptions {
    pub fn builder() -> ImageListOptionsBuilder {
        ImageListOptionsBuilder::default()
    }
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

/// Builder interface for `ImageListOptions`
#[derive(Default)]
pub struct ImageListOptionsBuilder {
    params: HashMap<&'static str, String>,
}

impl ImageListOptionsBuilder {
    pub fn digests(
        &mut self,
        d: bool,
    ) -> &mut Self {
        self.params.insert("digests", d.to_string());
        self
    }

    pub fn all(&mut self) -> &mut Self {
        self.params.insert("all", "true".to_owned());
        self
    }

    pub fn filter_name(
        &mut self,
        name: &str,
    ) -> &mut Self {
        self.params.insert("filter", name.to_owned());
        self
    }

    pub fn filter(
        &mut self,
        filters: Vec<ImageFilter>,
    ) -> &mut Self {
        let mut param = HashMap::new();
        for f in filters {
            match f {
                ImageFilter::Dangling => param.insert("dangling", vec![true.to_string()]),
                ImageFilter::LabelName(n) => param.insert("label", vec![n]),
                ImageFilter::Label(n, v) => param.insert("label", vec![format!("{}={}", n, v)]),
            };
        }
        // structure is a a json encoded object mapping string keys to a list
        // of string values
        self.params
            .insert("filters", serde_json::to_string(&param).unwrap());
        self
    }

    pub fn build(&self) -> ImageListOptions {
        ImageListOptions {
            params: self.params.clone(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub description: String,
    pub is_official: bool,
    pub is_automated: bool,
    pub name: String,
    pub star_count: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ImageInfo {
    #[cfg(feature = "chrono")]
    #[serde(deserialize_with = "datetime_from_unix_timestamp")]
    pub created: DateTime<Utc>,
    #[cfg(not(feature = "chrono"))]
    pub created: u64,
    pub id: String,
    pub parent_id: String,
    pub labels: Option<HashMap<String, String>>,
    pub repo_tags: Option<Vec<String>>,
    pub repo_digests: Option<Vec<String>>,
    pub virtual_size: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ImageDetails {
    pub architecture: String,
    pub author: String,
    pub comment: String,
    pub config: Config,
    #[cfg(feature = "chrono")]
    pub created: DateTime<Utc>,
    #[cfg(not(feature = "chrono"))]
    pub created: String,
    pub docker_version: String,
    pub id: String,
    pub os: String,
    pub parent: String,
    pub repo_tags: Option<Vec<String>>,
    pub repo_digests: Option<Vec<String>>,
    pub size: u64,
    pub virtual_size: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Config {
    pub attach_stderr: bool,
    pub attach_stdin: bool,
    pub attach_stdout: bool,
    pub cmd: Option<Vec<String>>,
    pub domainname: String,
    pub entrypoint: Option<Vec<String>>,
    pub env: Option<Vec<String>>,
    pub exposed_ports: Option<HashMap<String, HashMap<String, String>>>,
    pub hostname: String,
    pub image: String,
    pub labels: Option<HashMap<String, String>>,
    // pub MacAddress: String,
    pub on_build: Option<Vec<String>>,
    // pub NetworkDisabled: bool,
    pub open_stdin: bool,
    pub stdin_once: bool,
    pub tty: bool,
    pub user: String,
    pub working_dir: String,
}

impl Config {
    pub fn env(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        if let Some(ref vars) = self.env {
            for e in vars {
                let pair: Vec<&str> = e.split('=').collect();
                map.insert(pair[0].to_owned(), pair[1].to_owned());
            }
        }
        map
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct History {
    pub id: String,
    #[cfg(feature = "chrono")]
    #[serde(deserialize_with = "datetime_from_unix_timestamp")]
    pub created: DateTime<Utc>,
    #[cfg(not(feature = "chrono"))]
    pub created: u64,
    pub created_by: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Status {
    Untagged(String),
    Deleted(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test registry auth with token
    #[test]
    fn registry_auth_token() {
        let options = RegistryAuth::token("abc");
        assert_eq!(
            base64::encode(r#"{"identitytoken":"abc"}"#),
            options.serialize()
        );
    }

    /// Test registry auth with username and password
    #[test]
    fn registry_auth_password_simple() {
        let options = RegistryAuth::builder()
            .username("user_abc")
            .password("password_abc")
            .build();
        assert_eq!(
            base64::encode(r#"{"username":"user_abc","password":"password_abc"}"#),
            options.serialize()
        );
    }

    /// Test registry auth with all fields
    #[test]
    fn registry_auth_password_all() {
        let options = RegistryAuth::builder()
            .username("user_abc")
            .password("password_abc")
            .email("email_abc")
            .server_address("https://example.org")
            .build();
        assert_eq!(
            base64::encode(
                r#"{"username":"user_abc","password":"password_abc","email":"email_abc","serveraddress":"https://example.org"}"#
            ),
            options.serialize()
        );
    }
}
