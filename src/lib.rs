//! infinispan-rs is a Rust client for the [Infinispan REST
//! API](https://infinispan.org/docs/stable/titles/rest/rest.html).
//!
//! # Basic operation
//!
//! ```no_run
//! use infinispan::Infinispan;
//! use infinispan::errors::InfinispanError;
//! use infinispan::request;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create a client
//!     let client = Infinispan::new("http://localhost:11222", "username", "password");
//!
//!     // Create a cache
//!     let req = request::caches::create_local("some_cache");
//!     let _ = client.run(&req).await.unwrap();
//!
//!     // Create an entry
//!     let req = request::entries::create("some_cache", "some_entry").with_value("a_value".into());
//!     let _ = client.run(&req).await.unwrap();
//!
//!     // Read the entry
//!     let req = request::entries::get("some_cache", "some_entry");
//!     let resp = client.run(&req).await.unwrap();
//!
//!     // resp is an instance of `reqwest::Response`
//!     assert!(resp.status().is_success());
//!     assert_eq!("a_value", resp.text_with_charset("utf-8").await.unwrap());
//! }
//!
//! ```
//!
//! infinispan-rs supports requests to manage Caches, Entries, and Counters, but
//! for now, it only implements a reduced subset of the REST API. Here are some
//! examples:
//!
//! ## Caches
//!
//! ```no_run
//! use infinispan::Infinispan;
//! use infinispan::request;
//!
//! #[tokio::main]
//! async fn main() {
//!     let client = Infinispan::new("http://localhost:11222", "username", "password");
//!
//!     // Create a cache
//!     let req = request::caches::create_local("some_cache");
//!     let _ = client.run(&req).await.unwrap();
//!
//!     // Delete a cache
//!     let req = request::caches::delete("some_cache");
//!     let _ = client.run(&req).await.unwrap();
//! }
//! ```
//!
//! ## Entries
//!
//! ```no_run
//! use infinispan::Infinispan;
//! use infinispan::request;
//! use std::time::Duration;
//! use http::StatusCode;
//!
//! #[tokio::main]
//! async fn main() {
//!     let client = Infinispan::new("http://localhost:11222", "username", "password");
//!
//!     // Create a cache
//!     let req = request::caches::create_local("some_cache");
//!     let _ = client.run(&req).await.unwrap();
//!
//!     // Create an entry
//!     let req = request::entries::create("some_cache", "some_entry").with_value("a_value".into());
//!     let _ = client.run(&req).await.unwrap();
//!
//!     // Create an entry with a value and TTL
//!     let req = request::entries::create("some_cache", "some_entry_with_ttl")
//!         .with_value("a_value".into()).with_ttl(Duration::from_secs(5));
//!     let _ = client.run(&req).await.unwrap();
//!
//!     // Check if an entry exists
//!     let req = request::entries::exists("some_cache", "some_entry");
//!     let resp = client.run(&req).await.unwrap();
//!     assert!(resp.status().is_success());
//!
//!     let req = request::entries::exists("some_cache", "non_existing");
//!     let resp = client.run(&req).await.unwrap();
//!     assert_eq!(StatusCode::NOT_FOUND, resp.status());
//!
//!     // Update an entry
//!     let req = request::entries::update("some_cache", "some_entry", "new_val");
//!     let _ = client.run(&req).await.unwrap();
//!
//!     // Delete an entry
//!     let req = request::entries::delete("some_cache", "some_entry");
//!     let _ = client.run(&req).await.unwrap();
//! }
//! ```
//!
//! ## Counters
//!
//! ```no_run
//! use infinispan::Infinispan;
//! use infinispan::request;
//!
//! #[tokio::main]
//! async fn main() {
//!     let client = Infinispan::new("http://localhost:11222", "username", "password");
//!
//!     // Create a weak counter
//!     let req = request::counters::create_weak("some_counter").with_value(100);
//!     let _ = client.run(&req).await.unwrap();
//!
//!     // Increment a counter
//!     let req = request::counters::increment("some_counter");
//!     let _ = client.run(&req).await.unwrap();
//!
//!     // Increment a counter by a given delta
//!     let req = request::counters::increment("some_counter").by(10);
//!     let _ = client.run(&req).await.unwrap();
//!
//!     // Delete a counter
//!     let req = request::counters::delete("some_counter");
//!     let _ = client.run(&req).await.unwrap();
//!
//!     // Create a strong counter
//!     let req = request::counters::create_strong("some_strong_counter");
//!     let _ = client.run(&req).await.unwrap();
//! }
//! ```
//!

#![deny(clippy::all, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

use diqwest::WithDigestAuth;

use reqwest::Response;

use crate::request::ToHttpRequest;

pub mod errors;
pub mod request;

#[derive(Debug, Clone)]
pub struct Infinispan {
    base_url: String,
    http_client: reqwest::Client,
    username: String,
    password: String,
}

impl Infinispan {
    pub fn new(base_url: impl Into<String>, username: &str, password: &str) -> Self {
        Self {
            base_url: base_url.into(),
            http_client: reqwest::Client::new(),
            username: username.to_string(),
            password: password.to_string(),
        }
    }

    pub async fn run<R: ToHttpRequest>(
        &self,
        request: &R,
    ) -> Result<Response, diqwest::error::Error> {
        let http_req = request.to_http_req(&self.base_url);
        let res = self
            .http_client
            .request(http_req.method().clone(), http_req.uri().to_string())
            .headers(http_req.headers().clone())
            .body(http_req.body().clone())
            .send_with_digest_auth(self.username.as_str(), self.password.as_str())
            .await?;
        Ok(res)
    }
}
