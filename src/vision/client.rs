use std::env;
use std::sync::{Arc, Mutex};

use http::HeaderValue;
use tonic::transport::{Certificate, Channel, ClientTlsConfig};

use crate::authorize::{ApplicationCredentials, TokenManager, TLS_CERTS};
use crate::vision::{api, Error, Image, TextAnnotation, TextDetectionConfig};

/// The Pub/Sub client, tied to a specific project.
pub struct Client {
    pub(crate) project_name: String,
    pub(crate) img_annotator: Mutex<api::client::ImageAnnotatorClient<Channel>>,
    pub(crate) product_search: Mutex<api::client::ProductSearchClient<Channel>>,
}

impl Client {
    pub(crate) const DOMAIN_NAME: &'static str = "vision.googleapis.com";
    pub(crate) const ENDPOINT: &'static str = "https://vision.googleapis.com";
    pub(crate) const SCOPES: [&'static str; 2] = [
        "https://www.googleapis.com/auth/cloud-platform",
        "https://www.googleapis.com/auth/cloud-vision",
    ];

    /// Create a new client for the specified project.
    ///
    /// Credentials are looked up in the `GOOGLE_APPLICATION_CREDENTIALS` environment variable.
    pub async fn new(project_name: impl Into<String>) -> Result<Client, Error> {
        let path = env::var("GOOGLE_APPLICATION_CREDENTIALS")?;
        let file = std::fs::File::open(path)?;
        let creds = json::from_reader(file)?;

        Client::from_credentials(project_name, creds).await
    }

    /// Create a new client for the specified project with custom credentials.
    pub async fn from_credentials(
        project_name: impl Into<String>,
        creds: ApplicationCredentials,
    ) -> Result<Client, Error> {
        let mut tls_config = ClientTlsConfig::with_rustls();
        tls_config.ca_certificate(Certificate::from_pem(TLS_CERTS));
        tls_config.domain_name(Client::DOMAIN_NAME);

        let token_manager = Arc::new(Mutex::new(TokenManager::new(
            creds,
            Client::SCOPES.as_ref(),
        )));

        let channel = Channel::from_static(Client::ENDPOINT)
            .intercept_headers(move |headers| {
                let mut manager = token_manager.lock().unwrap();
                let token = manager.token();
                let value = HeaderValue::from_str(token.as_str()).unwrap();
                headers.insert("authorization", value);
            })
            .tls_config(&tls_config)
            .connect()
            .await?;

        Ok(Client {
            project_name: project_name.into(),
            img_annotator: Mutex::new(api::client::ImageAnnotatorClient::new(channel.clone())),
            product_search: Mutex::new(api::client::ProductSearchClient::new(channel)),
        })
    }

    /// Perform text detection on the given image.
    pub async fn detect_document_text(
        &self,
        image: Image,
        config: TextDetectionConfig,
    ) -> Result<Vec<TextAnnotation>, Error> {
        let mut service = self.img_annotator.lock().unwrap();

        let request = api::AnnotateImageRequest {
            image: Some(image.into()),
            features: vec![api::Feature {
                r#type: api::feature::Type::TextDetection as i32,
                max_results: 0, // Does not apply for TEXT_DETECTION, so set it to zero.
                model: String::from("builtin/stable"),
            }],
            image_context: Some(config.into()),
        };
        let request = api::BatchAnnotateImagesRequest {
            requests: vec![request],
            parent: String::default(), // TODO: Make this configurable (specifying computation region).
        };
        let response = service.batch_annotate_images(request).await?;
        let response = response.into_inner();
        let response = response.responses.into_iter().next().unwrap();
        let annotations = response
            .text_annotations
            .into_iter()
            .map(TextAnnotation::from)
            .collect();

        Ok(annotations)
    }
}
