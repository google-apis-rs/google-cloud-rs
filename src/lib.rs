#![warn(missing_docs)]
//! Asynchronous Rust bindings for Google Cloud Platform gRPC APIs.

mod utils;

/// Authorization/authentication related utilities.
pub mod authorize;

/// Pub/Sub bindings.
#[cfg(feature = "pubsub")]
pub mod pubsub;
/// Datastore bindings.
#[cfg(feature = "datastore")]
pub mod datastore;

#[cfg(test)]
mod tests;
