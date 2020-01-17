mod api;
mod bucket;
mod client;
mod object;

pub use self::bucket::*;
pub use self::client::*;
pub use self::object::*;

/// The error type for the Cloud Storage module.
pub type Error = crate::error::Error;
