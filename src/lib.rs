#![warn(missing_docs)]
//! Asynchronous Rust bindings for Google Cloud Platform gRPC APIs.

mod utils;

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
/// Cloud Vision bindings.
#[cfg(feature = "vision")]
pub mod vision;
/// Cloud Storage bindings.
#[cfg(feature = "storage")]
pub mod storage;

#[cfg(test)]
mod tests;
