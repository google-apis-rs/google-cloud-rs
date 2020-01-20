use crate::authorize::ApplicationCredentials;
use crate::storage;

async fn setup_client() -> Result<storage::Client, storage::Error> {
    let creds = json::from_str::<ApplicationCredentials>(env!("GCP_TEST_CREDENTIALS"))?;
    storage::Client::from_credentials(env!("GCP_TEST_PROJECT"), creds).await
}

#[tokio::test]
async fn storage_lists_buckets() {
    let client = setup_client().await;
    assert!(client.is_ok());
    let mut client = client.unwrap();
    let buckets = client.buckets().await;
    assert!(buckets.is_ok());
    let buckets = buckets.unwrap();
    for bucket in buckets.iter() {
        println!("bucket: {}", bucket.name);
    }
}

#[tokio::test]
async fn storage_create_and_delete_bucket() {
    let client = setup_client().await;
    assert!(client.is_ok());
    let mut client = client.unwrap();
    let bucket = match client.bucket(env!("GCP_TEST_BUCKET")).await {
        Ok(bucket) => Ok(bucket),
        Err(_) => client.create_bucket(env!("GCP_TEST_BUCKET")).await,
    };
    assert!(bucket.is_ok());
    let mut bucket = bucket.unwrap();
    println!("created bucket: {}", bucket.name());
    let object = bucket
        .create_object("sample.json", r#"{"type":"sample","from":"google-cloud"}"#)
        .await;
    let object = object.map_err(|err| {
        eprintln!("{}", err);
        err
    });
    assert!(object.is_ok());
    let object = object.unwrap();
    println!(
        "created object: {} (into: {})",
        object.name(),
        object.bucket(),
    );
    let result = object.delete().await;
    assert!(result.is_ok());
    let _ = result.unwrap();
    let result = bucket.delete().await;
    assert!(result.is_ok());
    let _ = result.unwrap();
}
