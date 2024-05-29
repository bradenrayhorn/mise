#![deny(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]

pub mod config;
pub mod core;
pub mod datastore;
pub mod domain;
pub mod http;
pub mod oidc;
pub mod session_store;
pub mod sqlite;
