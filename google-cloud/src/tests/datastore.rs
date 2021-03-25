use std::collections::HashMap;

use crate::datastore;
use crate::datastore::IntoValue;

macro_rules! assert_ok {
    ($expr:expr) => {
        match $expr {
            Ok(value) => value,
            Err(err) => {
                panic!("asserted result is an error: {}", err);
            }
        }
    };
}

async fn setup_client() -> Result<datastore::Client, datastore::Error> {
    // TODO env!("GCP_PROJECT")
    datastore::Client::new("dl-datastore-stage".to_string()).await
}

#[tokio::test]
async fn datastore_puts_data_successfully() {
    //? Setup test client.
    let mut client = assert_ok!(setup_client().await);

    //? Prepare Datastore key and value.
    let key = datastore::Key::new("google-cloud-tests")
        .namespace("test")
        .id("test-id");
    let properties = {
        let mut values = HashMap::new();
        values.insert(String::from("hello"), "world !".into_value());
        values.insert(
            String::from("time"),
            datastore::Value::TimestampValue(chrono::Local::now().naive_local()),
        );
        values
    };

    //? Store value in Datastore.
    assert_ok!(client.put((key.clone(), properties)).await);

    //? Get value back from Datastore.
    let outcome = assert_ok!(client.get::<datastore::Value, _>(&key).await);
    assert!(outcome.is_some());

    //? Delete that value from Datastore.
    assert_ok!(client.delete(key).await);
}
