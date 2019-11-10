use std::collections::HashMap;

use crate::authorize::ApplicationCredentials;
use crate::datastore;

async fn setup_client() -> datastore::Client {
    let creds = json::from_str::<ApplicationCredentials>(env!("GCP_TEST_CREDENTIALS"))
        .expect("invalid GCP credentials format");
    let client = datastore::Client::from_credentials(env!("GCP_TEST_PROJECT"), creds).await;

    client.expect("could not create datastore client")
}

#[tokio::test]
async fn datastore_puts_data_successfully() {
    let client = setup_client().await;
    let key = datastore::Key::new("gcp-rs-tests").namespace("test").id(4);
    let properties = {
        let mut values = HashMap::new();
        values.insert("hello".into(), "world !".into());
        values.insert(
            "time".into(),
            datastore::Value::TimestampValue(chrono::Local::now().naive_local()),
        );
        values
    };
    let entity = datastore::Entity::new(key.clone(), properties);
    let outcome = client.put(entity).await;
    assert!(outcome.is_ok());
    let outcome = client.delete(key).await;
    assert!(outcome.is_ok());
}
