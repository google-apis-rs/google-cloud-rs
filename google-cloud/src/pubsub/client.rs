use std::env;
use std::fs::File;
use std::sync::Arc;

use tokio::sync::Mutex;
use tonic::transport::{Certificate, Channel, ClientTlsConfig};
use tonic::{IntoRequest, Request};

use crate::authorize::{ApplicationCredentials, TokenManager, TLS_CERTS};
use crate::pubsub::api;
use crate::pubsub::api::publisher_client::PublisherClient;
use crate::pubsub::api::subscriber_client::SubscriberClient;
use crate::pubsub::{Error, Subscription, Topic, TopicConfig};

/// The Pub/Sub client, tied to a specific project.
#[derive(Clone)]
pub struct Client {
    pub(crate) project_name: String,
    pub(crate) publisher: PublisherClient<Channel>,
    pub(crate) subscriber: SubscriberClient<Channel>,
    pub(crate) token_manager: Arc<Mutex<TokenManager>>,
}

impl Client {
    pub(crate) const DOMAIN_NAME: &'static str = "pubsub.googleapis.com";
    pub(crate) const ENDPOINT: &'static str = "https://pubsub.googleapis.com";
    pub(crate) const SCOPES: [&'static str; 2] = [
        "https://www.googleapis.com/auth/cloud-platform",
        "https://www.googleapis.com/auth/pubsub",
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

        let channel = Channel::from_static(Client::ENDPOINT)
            .tls_config(tls_config)?
            .connect()
            .await?;

        Ok(Client {
            project_name: project_name.into(),
            publisher: PublisherClient::new(channel.clone()),
            subscriber: SubscriberClient::new(channel),
            token_manager: Arc::new(Mutex::new(TokenManager::new(
                creds,
                Client::SCOPES.as_ref(),
            ))),
        })
    }

    /// Create a new topic.
    pub async fn create_topic(
        &mut self,
        topic_name: &str,
        config: TopicConfig,
    ) -> Result<Topic, Error> {
        let request = api::Topic {
            name: format!(
                "projects/{0}/topics/{1}",
                self.project_name.as_str(),
                topic_name,
            ),
            labels: config.labels,
            message_storage_policy: None,
            kms_key_name: String::new(),
        };
        let request = self.construct_request(request).await?;
        let response = self.publisher.create_topic(request).await?;
        let topic = response.into_inner();
        let name = topic.name.split('/').last().unwrap_or(topic_name);

        Ok(Topic::new(self.clone(), name))
    }

    /// List all exisiting topics.
    pub async fn topics(&mut self) -> Result<Vec<Topic>, Error> {
        let mut topics = Vec::new();
        let page_size = 25;
        let mut page_token = String::default();

        loop {
            let request = api::ListTopicsRequest {
                project: format!("projects/{0}", self.project_name.as_str()),
                page_size,
                page_token,
            };
            let request = self.construct_request(request).await?;
            let response = self.publisher.list_topics(request).await?;
            let response = response.into_inner();
            page_token = response.next_page_token;
            topics.extend(response.topics.into_iter().map(|topic| {
                let name = topic.name.split('/').last().unwrap();
                Topic::new(self.clone(), name)
            }));
            if page_token.is_empty() {
                break;
            }
        }

        Ok(topics)
    }

    /// Get a handle to a specific topic.
    pub async fn topic(&mut self, topic_name: &str) -> Result<Option<Topic>, Error> {
        let request = api::GetTopicRequest {
            topic: format!(
                "projects/{0}/topics/{1}",
                self.project_name.as_str(),
                topic_name
            ),
        };
        let request = self.construct_request(request).await?;
        let response = self.publisher.get_topic(request).await?;
        let topic = response.into_inner();
        let name = topic.name.split('/').last().unwrap_or(topic_name);

        Ok(Some(Topic::new(self.clone(), name)))
    }

    /// List all existing subscriptions (to any topic).
    pub async fn subscriptions(&mut self) -> Result<Vec<Subscription>, Error> {
        let mut subscriptions = Vec::new();
        let page_size = 25;
        let mut page_token = String::default();

        loop {
            let request = api::ListSubscriptionsRequest {
                project: format!("projects/{0}", self.project_name.as_str()),
                page_size,
                page_token,
            };
            let request = self.construct_request(request).await?;
            let response = self.subscriber.list_subscriptions(request).await?;
            let response = response.into_inner();
            page_token = response.next_page_token;
            subscriptions.extend(response.subscriptions.into_iter().map(|subscription| {
                let name = subscription.name.split('/').last().unwrap();
                Subscription::new(self.clone(), name)
            }));
            if page_token.is_empty() {
                break;
            }
        }

        Ok(subscriptions)
    }

    /// Get a handle of a specific subscription.
    pub async fn subscription(
        &mut self,
        subscription_name: &str,
    ) -> Result<Option<Subscription>, Error> {
        let request = api::GetSubscriptionRequest {
            subscription: format!(
                "projects/{0}/subscriptions/{1}",
                self.project_name.as_str(),
                subscription_name
            ),
        };
        let request = self.construct_request(request).await?;
        let response = self.subscriber.get_subscription(request).await?;
        let subscription = response.into_inner();
        let name = subscription.name.split('/').last();
        let name = name.unwrap_or(subscription_name);

        Ok(Some(Subscription::new(self.clone(), name)))
    }
}
