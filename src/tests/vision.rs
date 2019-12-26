use crate::authorize::ApplicationCredentials;
use crate::vision;

async fn setup_client() -> Result<vision::Client, vision::Error> {
    let creds = json::from_str::<ApplicationCredentials>(env!("GCP_TEST_CREDENTIALS"))?;
    vision::Client::from_credentials(env!("GCP_TEST_PROJECT"), creds).await
}

#[tokio::test]
async fn vision_connects_successfully() {
    let client = setup_client().await;
    assert!(client.is_ok());
}

#[tokio::test]
async fn vision_detects_text_successfully() {
    let client = setup_client().await;
    assert!(client.is_ok());
    let mut client = client.unwrap();

    let bytes = tokio::fs::read("samples/placeholder.png").await.unwrap();
    let image = vision::Image::from_bytes(bytes);

    //? Image from URL disabled due to GCP bug.
    //? More informations here:
    //? https://github.com/googleapis/nodejs-vision/issues/270#issuecomment-481064953
    // let image = vision::Image::from_url("https://placehold.it/500");

    let config = vision::TextDetectionConfig::default();
    let detected = client.detect_document_text(image, config).await;
    assert!(dbg!(detected).is_ok());
}
