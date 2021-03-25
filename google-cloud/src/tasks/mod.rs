mod client;
mod queue;
mod task;
mod task_authorization;
mod task_request_types;
mod utils;
mod api {
    pub mod google {
        pub mod api {
            include!("api/google.api.rs");
        }
        pub mod cloud {
            pub mod tasks {
                pub mod v2beta3 {
                    include!("api/google.cloud.tasks.v2beta3.rs");
                }
            }
        }
        pub mod iam {
            pub mod v1 {
                include!("api/google.iam.v1.rs");
            }
        }
        pub mod protobuf {
            include!("api/google.protobuf.rs");
        }
        pub mod r#type {
            include!("api/google.r#type.rs");
        }
        pub mod rpc {
            include!("api/google.rpc.rs");
        }
    }
    pub use self::google::cloud::tasks::v2beta3::*;
}

pub use self::client::*;
pub use self::queue::*;
pub use self::task::*;
pub use self::task_authorization::*;
pub use self::task_request_types::*;
pub(crate) use self::utils::*;

/// The error type for the Tasks module.
pub type Error = crate::error::Error;
