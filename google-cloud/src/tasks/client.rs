use std::env;
use std::fs::File;
use std::sync::Arc;

use tokio::sync::Mutex;
use tonic::transport::{Certificate, Channel, ClientTlsConfig};
use tonic::{IntoRequest, Request};
use tonic::metadata::MetadataValue;

use crate::authorize::{ApplicationCredentials, TokenManager, TLS_CERTS};
use crate::tasks::api;
use crate::tasks::api::cloud_tasks_client::CloudTasksClient;
use crate::tasks::{Error, Queue};

const ROUTING_METADATA_KEY: &str = "x-goog-request-params";

/// The Cloud Tasks client, tied to a specific project and location.
#[derive(Clone)]
pub struct Client {
    pub(crate) project_name: String,
    pub(crate) location_id: String,
    pub(crate) service: CloudTasksClient<Channel>,
    pub(crate) token_manager: Arc<Mutex<TokenManager>>,
}

impl Client {
    pub(crate) const DOMAIN_NAME: &'static str = "cloudtasks.googleapis.com";
    pub(crate) const ENDPOINT: &'static str = "https://cloudtasks.googleapis.com";
    pub(crate) const SCOPES: [&'static str; 2] = [
        "https://www.googleapis.com/auth/cloud-platform",
        "https://www.googleapis.com/auth/cloud-tasks",
    ];

    pub(crate) async fn construct_request<T: IntoRequest<T>>(
        &mut self,
        request: T,
    ) -> Result<Request<T>, Error> {
        let mut request = request.into_request();
        let token = self.token_manager.lock().await.token().await?;
        let metadata = request.metadata_mut();
        metadata.insert("authorization", token.parse().unwrap());
        Ok(request)
    }

    /// Create a new client for the specified project.
    ///
    /// Credentials are looked up in the `GOOGLE_APPLICATION_CREDENTIALS` environment variable.
    pub async fn new(project_name: impl Into<String>, location_id: impl Into<String>) -> Result<Client, Error> {
        let path = env::var("GOOGLE_APPLICATION_CREDENTIALS")?;
        let file = File::open(path)?;
        let creds = json::from_reader(file)?;

        Client::from_credentials(project_name, location_id, creds).await
    }

    /// Create a new client for the specified project with custom credentials.
    pub async fn from_credentials(
        project_name: impl Into<String>,
        location_id: impl Into<String>,
        creds: ApplicationCredentials,
    ) -> Result<Client, Error> {
        let tls_config = ClientTlsConfig::new()
            .ca_certificate(Certificate::from_pem(TLS_CERTS))
            .domain_name(Client::DOMAIN_NAME);

        let channel = Channel::from_static(Client::ENDPOINT)
            .tls_config(tls_config)?
            .connect()
            .await?;

        Ok(Client {
            project_name: project_name.into(),
            location_id: location_id.into(),
            service: CloudTasksClient::new(channel),
            token_manager: Arc::new(Mutex::new(TokenManager::new(
                creds,
                Client::SCOPES.as_ref(),
            ))),
        })
    }

    /// List queues
    /// `filter` argument allows returning only a subset of queues, sample filter: "state: PAUSED"
    pub async fn queues(&mut self, filter: &str) -> Result<Vec<Queue>, Error> {
        let mut queues = Vec::new();
        let page_size = 25;
        let mut page_token = String::default();

        loop {
            let request = api::ListQueuesRequest {
                parent: format!("projects/{0}/locations/{1}", self.project_name.as_str(), self.location_id.as_str()),
                filter: filter.to_string(),
                page_size,
                page_token,
            };
            let request = self.construct_request(request).await?;
            let response = self.service.list_queues(request).await?;
            let response = response.into_inner();
            page_token = response.next_page_token;
            queues.extend(
                response
                    .queues
                    .into_iter()
                    .map(|queue| Queue::new(self.clone(), queue.name)),
            );
            if page_token.is_empty() {
                break;
            }
        }

        Ok(queues)
    }

    /// Get a queue by name.
    pub async fn queue(&mut self, id: &str) -> Result<Option<Queue>, Error> {
        let name = format!(
            "projects/{0}/locations/{1}/queues/{2}",
            self.project_name.as_str(),
            self.location_id.as_str(),
            id,
        );
        let request = api::GetQueueRequest {
            name: name.clone(),
        };
        let mut request = self.construct_request(request).await?;
        // Add routing metadata
        request.metadata_mut().insert(ROUTING_METADATA_KEY, MetadataValue::from_str(format!("name={}", name).as_str()).unwrap());
        let response = self.service.get_queue(request).await?;
        let queue = response.into_inner();

        Ok(Some(Queue::new(self.clone(), queue.name)))
    }
}
