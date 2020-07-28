#![warn(missing_docs)]
//! Asynchronous Rust bindings for Google Cloud Platform gRPC APIs.

#[cfg(feature = "google-cloud-derive")]
extern crate google_cloud_derive;

/// Authorization/authentication related utilities.
pub mod authorize;
/// Error handling utilities.
pub mod error;

/// Datastore bindings.
#[cfg(feature = "datastore")]
pub mod datastore;
/// Pub/Sub bindings.
#[cfg(feature = "pubsub")]
pub mod pubsub;
/// Cloud Storage bindings.
#[cfg(feature = "storage")]
pub mod storage;
/// Cloud Vision bindings.
#[cfg(feature = "vision")]
pub mod vision;

#[cfg(test)]
mod tests;
