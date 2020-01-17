use std::convert::TryFrom;
use std::env;
use std::fs::File;
use std::sync::{Arc, Mutex};

use tonic::transport::{Certificate, Channel, ClientTlsConfig};
use tonic::{Request, Status};

use crate::authorize::{ApplicationCredentials, TokenManager, TLS_CERTS};
use crate::vision::api;
use crate::vision::api::image_annotator_client::ImageAnnotatorClient;
use crate::vision::api::product_search_client::ProductSearchClient;
use crate::vision::{
    Error, FaceAnnotation, FaceDetectionConfig, Image, TextAnnotation, TextDetectionConfig,
};

/// The Cloud Vision client, tied to a specific project.
#[derive(Clone)]
pub struct Client {
    pub(crate) project_name: String,
    pub(crate) img_annotator: ImageAnnotatorClient<Channel>,
    pub(crate) product_search: ProductSearchClient<Channel>,
}

impl Client {
    pub(crate) const DOMAIN_NAME: &'static str = "vision.googleapis.com";
    pub(crate) const ENDPOINT: &'static str = "https://vision.googleapis.com";
    pub(crate) const SCOPES: [&'static str; 2] = [
        "https://www.googleapis.com/auth/cloud-platform",
        "https://www.googleapis.com/auth/cloud-vision",
    ];

    pub(crate) fn interceptor(
        token_manager: Arc<Mutex<TokenManager>>,
    ) -> impl Fn(Request<()>) -> Result<Request<()>, Status> {
        move |mut request: Request<()>| {
            let mut manager = token_manager.lock().unwrap();
            let token = manager.token();
            let metadata = request.metadata_mut();
            metadata.insert("authorization", token.parse().unwrap());
            Ok(request)
        }
    }

    /// Create a new client for the specified project.
    ///
    /// Credentials are looked up in the `GOOGLE_APPLICATION_CREDENTIALS` environment variable.
    pub async fn new(project_name: impl Into<String>) -> Result<Client, Error> {
        let path = env::var("GOOGLE_APPLICATION_CREDENTIALS")?;
        let file = File::open(path)?;
        let creds = json::from_reader(file)?;

        Client::from_credentials(project_name, creds).await
    }

    /// Create a new client for the specified project with custom credentials.
    pub async fn from_credentials(
        project_name: impl Into<String>,
        creds: ApplicationCredentials,
    ) -> Result<Client, Error> {
        let tls_config = ClientTlsConfig::new()
            .ca_certificate(Certificate::from_pem(TLS_CERTS))
            .domain_name(Client::DOMAIN_NAME);

        let token_manager = Arc::new(Mutex::new(TokenManager::new(
            creds,
            Client::SCOPES.as_ref(),
        )));

        let channel = Channel::from_static(Client::ENDPOINT)
            .tls_config(tls_config)
            .connect()
            .await?;

        Ok(Client {
            project_name: project_name.into(),
            img_annotator: ImageAnnotatorClient::with_interceptor(
                channel.clone(),
                Client::interceptor(token_manager.clone()),
            ),
            product_search: ProductSearchClient::with_interceptor(
                channel,
                Client::interceptor(token_manager),
            ),
        })
    }

    /// Perform text detection on the given image.
    pub async fn detect_document_text(
        &mut self,
        image: Image,
        config: TextDetectionConfig,
    ) -> Result<Vec<TextAnnotation>, Error> {
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
        let response = self.img_annotator.batch_annotate_images(request).await?;
        let response = response.into_inner();
        let response = response.responses.into_iter().next().unwrap();
        let annotations = response
            .text_annotations
            .into_iter()
            .map(TextAnnotation::from)
            .collect();

        Ok(annotations)
    }

    /// Perform text detection on the given image.
    pub async fn detect_faces(
        &mut self,
        image: Image,
        config: FaceDetectionConfig,
    ) -> Result<Vec<FaceAnnotation>, Error> {
        let request = api::AnnotateImageRequest {
            image: Some(image.into()),
            features: vec![api::Feature {
                r#type: api::feature::Type::FaceDetection as i32,
                max_results: config.max_results,
                model: String::from("builtin/stable"),
            }],
            image_context: None,
        };
        let request = api::BatchAnnotateImagesRequest {
            requests: vec![request],
            parent: String::default(), // TODO: Make this configurable (specifying computation region).
        };
        let response = self.img_annotator.batch_annotate_images(request).await?;
        let response = response.into_inner();
        let response = response.responses.into_iter().next().unwrap();
        let annotations = response
            .face_annotations
            .into_iter()
            .flat_map(FaceAnnotation::try_from)
            .collect();

        Ok(annotations)
    }
}
