use std::env;
use std::fs::File;
use std::sync::Arc;

use json::json;

use crate::authorize::{ApplicationCredentials, TokenManager};
use crate::storage::api::bucket::{BucketResource, BucketResources};
use crate::storage::{Bucket, Error};

/// The Cloud Storage client, tied to a specific project.
#[derive(Clone)]
pub struct Client {
    pub(crate) project_name: String,
    pub(crate) client: Arc<reqwest::Client>,
    pub(crate) token_manager: TokenManager,
}

impl Client {
    pub(crate) const DOMAIN_NAME: &'static str = "storage.googleapis.com";
    pub(crate) const ENDPOINT: &'static str = "https://storage.googleapis.com/storage/v1";
    /// Cloud Storage uses a slightly different endpoint for uploads.
    pub(crate) const UPLOAD_ENDPOINT: &'static str =
        "https://storage.googleapis.com/upload/storage/v1";
    pub(crate) const SCOPES: [&'static str; 2] = [
        "https://www.googleapis.com/auth/cloud-platform",
        "https://www.googleapis.com/auth/devstorage.full_control",
    ];
    #[allow(dead_code)]
    pub(crate) fn uri(uri: &str) -> String {
        if uri.starts_with('/') {
            format!("{}{}", Client::ENDPOINT, uri)
        } else {
            format!("{}/{}", Client::ENDPOINT, uri)
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
        let token_manager = TokenManager::new(creds, Client::SCOPES.as_ref());
        // let certificate = reqwest::Certificate::from_pem(TLS_CERTS)?;
        let client = reqwest::Client::builder()
            // .add_root_certificate(certificate)
            .build()?;

        Ok(Client {
            token_manager,
            client: Arc::new(client),
            project_name: project_name.into(),
        })
    }

    /// Get a handle to a specific bucket.
    pub async fn bucket(&mut self, name: &str) -> Result<Bucket, Error> {
        let inner = &self.client;
        let uri = format!("{}/b/{}", Client::ENDPOINT, name);

        let request = inner
            .get(uri.as_str())
            .header("authorization", self.token_manager.token())
            .send();
        let response = request.await?;
        let bucket = response
            .error_for_status()?
            .json::<BucketResource>()
            .await?;

        Ok(Bucket::new(self.clone(), bucket.name))
    }

    /// List all existing buckets of the current project.
    pub async fn buckets(&mut self) -> Result<Vec<Bucket>, Error> {
        let inner = &self.client;
        let uri = format!("{}/b", Client::ENDPOINT);

        let request = inner
            .get(uri.as_str())
            .query(&[("project", self.project_name.as_str())])
            .header("authorization", self.token_manager.token())
            .send();
        let response = request.await?;
        let resources = response
            .error_for_status()?
            .json::<BucketResources>()
            .await?;

        let buckets = resources
            .items
            .into_iter()
            .map(|resource| Bucket::new(self.clone(), resource.name))
            .collect();

        Ok(buckets)
    }

    /// Create a new bucket and get a handle to it.
    pub async fn create_bucket(&mut self, name: &str) -> Result<Bucket, Error> {
        let inner = &self.client;
        let uri = format!("{}/b", Client::ENDPOINT);

        let body = json!({
            "kind": "storage#bucket",
            "name": name,
        });
        let request = inner
            .post(uri.as_str())
            .query(&[("project", self.project_name.as_str())])
            .header("authorization", self.token_manager.token())
            .json(&body)
            .send();
        let response = request.await?;
        let bucket = response
            .error_for_status()?
            .json::<BucketResource>()
            .await?;

        Ok(Bucket::new(self.clone(), bucket.name))
    }
}
