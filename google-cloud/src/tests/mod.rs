#[cfg(feature = "datastore")]
mod datastore;
#[cfg(feature = "pubsub")]
mod pubsub;
#[cfg(feature = "storage")]
mod storage;
#[cfg(feature = "vision")]
mod vision;

use crate::authorize::ApplicationCredentials;

fn load_creds() -> ApplicationCredentials {
    let creds = std::env::var("GCP_TEST_CREDENTIALS").expect("env GCP_TEST_CREDENTIALS not set");
    json::from_str::<ApplicationCredentials>(&creds)
        .expect("incorrect application credentials format")
}
