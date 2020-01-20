mod client;
mod message;
mod subscription;
mod topic;
mod api {
    include!("api/google.pubsub.v1.rs");
}

pub use self::client::*;
pub use self::message::*;
pub use self::subscription::*;
pub use self::topic::*;

/// The error type for the PubSub module.
pub type Error = crate::error::Error;
