mod client;
mod task;
mod queue;
mod api {
    #[path = "google.cloud.tasks.v2beta3.rs"]
    mod tasks_api;
    pub use self::tasks_api::*;
}

pub use self::client::*;
pub use self::task::*;
pub use self::queue::*;
pub use self::topic::*;

/// The error type for the Tasks module.
pub type Error = crate::error::Error;
