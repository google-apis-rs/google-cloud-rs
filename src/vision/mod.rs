mod bounding_box;
mod client;
mod image;
mod text;
mod api {
    pub mod rpc {
        include!("api/google.rpc.rs");
    }
    #[allow(clippy::module_inception)]
    pub mod api {
        include!("api/google.api.rs");
    }
    pub mod longrunning {
        include!("api/google.longrunning.rs");
    }
    pub mod protobuf {
        include!("api/google.protobuf.rs");
    }
    pub mod r#type {
        include!("api/google.r#type.rs");
    }
    pub mod cloud {
        pub mod vision {
            pub mod v1 {
                include!("api/google.cloud.vision.v1.rs");
            }
        }
    }
    pub use self::cloud::vision::v1::*;
    pub use self::r#type::*;
}

pub use self::bounding_box::*;
pub use self::client::*;
pub use self::image::*;
pub use self::text::*;

/// The error type for the Cloud Vision client.
pub type Error = Box<dyn std::error::Error + 'static>;
