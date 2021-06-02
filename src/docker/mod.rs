
use http_types::{Mime};


pub mod docker;
pub mod image;
pub mod container;
// pub mod exec;
pub mod network;
pub mod service;
pub mod volume;

pub mod tarball;



pub fn tar() -> Mime {
    "application/tar".parse().unwrap()
}