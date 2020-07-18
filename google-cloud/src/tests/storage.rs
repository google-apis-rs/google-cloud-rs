use crate::authorize::ApplicationCredentials;
use crate::storage;

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

async fn setup_client() -> Result<storage::Client, storage::Error> {
    let creds = json::from_str::<ApplicationCredentials>(env!("GCP_TEST_CREDENTIALS"))?;
    storage::Client::from_credentials(env!("GCP_TEST_PROJECT"), creds).await
}

#[tokio::test]
async fn storage_lists_buckets() {
    //? Setup test client.
    let mut client = assert_ok!(setup_client().await);

    //? List all buckets of the project.
    let buckets = assert_ok!(client.buckets().await);

    //? Print their names to stdout.
    for bucket in buckets.iter() {
        println!("bucket: {}", bucket.name);
    }
}

#[tokio::test]
async fn storage_create_and_delete_bucket() {
    //? Setup test client.
    let mut client = assert_ok!(setup_client().await);

    //? Access existing bucket or create it, if non-existant.
    let bucket = match client.bucket(env!("GCP_TEST_BUCKET")).await {
        Ok(bucket) => Ok(bucket),
        Err(_) => client.create_bucket(env!("GCP_TEST_BUCKET")).await,
    };
    let mut bucket = assert_ok!(bucket);
    println!("got bucket: {}", bucket.name());

    //? Access existing object in that bucket or create it, if non-existant.
    let object_data = r#"{"type":"sample","from":"google-cloud-rs"}"#;
    let object = match bucket.object(env!("GCP_TEST_OBJECT")).await {
        Ok(object) => Ok(object),
        Err(_) => {
            bucket
                .create_object(env!("GCP_TEST_OBJECT"), object_data, "application/json")
                .await
        }
    };
    let mut object = assert_ok!(object);
    println!("got object: {} (into: {})", object.name(), object.bucket());

    //? Read the object's data back.
    let data = assert_ok!(object.get().await);
    let expected: json::Value = assert_ok!(json::from_str(object_data));
    let got: json::Value = assert_ok!(json::from_slice(data.as_slice()));
    assert_eq!(expected, got);
    println!("object contents are identical.");

    //? Delete that object.
    assert_ok!(object.delete().await);

    //? Delete the bucket.
    assert_ok!(bucket.delete().await);
}
