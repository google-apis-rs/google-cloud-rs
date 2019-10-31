#![warn(missing_docs)]
//! Asynchronous Rust bindings for Google Cloud Platform gRPC APIs.

mod utils;

/// Authorization/authentication related utilities.
pub mod authorize;

/// Pub/Sub bindings.
#[cfg(feature = "pubsub")]
pub mod pubsub;

#[cfg(test)]
mod tests;
