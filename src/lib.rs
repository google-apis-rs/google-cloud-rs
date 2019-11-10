#![warn(missing_docs)]
//! Asynchronous Rust bindings for Google Cloud Platform gRPC APIs.

mod utils;

/// Authorization/authentication related utilities.
pub mod authorize;

/// Datastore bindings.
#[cfg(feature = "datastore")]
pub mod datastore;
/// Pub/Sub bindings.
#[cfg(feature = "pubsub")]
pub mod pubsub;

#[cfg(test)]
mod tests;
