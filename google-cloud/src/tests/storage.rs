use crate::authorize::ApplicationCredentials;
use crate::storage;

async fn setup_client() -> Result<storage::Client, storage::Error> {
    let creds = json::from_str::<ApplicationCredentials>(env!("GCP_TEST_CREDENTIALS"))?;
    storage::Client::from_credentials(env!("GCP_TEST_PROJECT"), creds).await
}

#[tokio::test]
async fn storage_lists_buckets() {
    //? Setup test client.
    let client = setup_client().await;
    assert!(client.is_ok());
    let mut client = client.unwrap();

    //? List all buckets of the project.
    let buckets = client.buckets().await;
    assert!(buckets.is_ok());
    let buckets = buckets.unwrap();

    //? Print their names to stdout.
    for bucket in buckets.iter() {
        println!("bucket: {}", bucket.name);
    }
}

#[tokio::test]
async fn storage_create_and_delete_bucket() {
    //? Setup test client.
    let client = setup_client().await;
    assert!(client.is_ok());
    let mut client = client.unwrap();

    //? Access existing bucket or create it, if non-existant.
    let bucket = match client.bucket(env!("GCP_TEST_BUCKET")).await {
        Ok(bucket) => Ok(bucket),
        Err(_) => client.create_bucket(env!("GCP_TEST_BUCKET")).await,
    };
    assert!(bucket.is_ok());
    let mut bucket = bucket.unwrap();
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
    assert!(object.is_ok());
    let mut object = object.unwrap();
    println!("got object: {} (into: {})", object.name(), object.bucket());

    //? Read the object's data back.
    let data = object.read().await;
    assert!(data.is_ok());
    let data = data.unwrap();
    let expected: json::Value = json::from_str(object_data).unwrap();
    let got: Result<json::Value, json::Error> = json::from_slice(data.as_slice());
    assert!(got.is_ok());
    let got = got.unwrap();
    assert_eq!(expected, got);
    println!("object contents is identical.");

    //? Delete that object.
    let result = object.delete().await;
    assert!(result.is_ok());
    let _ = result.unwrap();

    //? Delete the bucket.
    let result = bucket.delete().await;
    assert!(result.is_ok());
    let _ = result.unwrap();
}
