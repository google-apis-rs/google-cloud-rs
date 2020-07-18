use crate::authorize::ApplicationCredentials;
use crate::vision;

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

async fn setup_client() -> Result<vision::Client, vision::Error> {
    let creds = json::from_str::<ApplicationCredentials>(env!("GCP_TEST_CREDENTIALS"))?;
    vision::Client::from_credentials(env!("GCP_TEST_PROJECT"), creds).await
}

#[tokio::test]
async fn vision_connects_successfully() {
    assert_ok!(setup_client().await);
}

#[tokio::test]
async fn vision_detects_text_successfully() {
    let mut client = assert_ok!(setup_client().await);

    let bytes = assert_ok!(tokio::fs::read("samples/placeholder.png").await);
    let image = vision::Image::from_bytes(bytes);

    //? Image from URL disabled due to GCP bug.
    //? More informations here:
    //? https://github.com/googleapis/nodejs-vision/issues/270#issuecomment-481064953
    // let image = vision::Image::from_url("https://placehold.it/500");

    let config = vision::TextDetectionConfig::default();
    assert_ok!(client.detect_document_text(image, config).await);
}
