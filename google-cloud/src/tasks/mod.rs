mod task_request_types;
mod task_authorization;
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
pub use self::task_authorization::*;
pub use self::task_request_types::*;

/// The error type for the Tasks module.
pub type Error = crate::error::Error;
