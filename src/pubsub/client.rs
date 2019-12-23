use std::env;
use std::sync::{Arc, Mutex};

use http::HeaderValue;
use tonic::transport::{Certificate, Channel, ClientTlsConfig};

use crate::authorize::{ApplicationCredentials, TokenManager, TLS_CERTS};
use crate::pubsub::{api, Error, Subscription, Topic, TopicConfig};

/// The Pub/Sub client, tied to a specific project.
pub struct Client {
    pub(crate) project_name: String,
    pub(crate) publisher: Mutex<api::publisher_client::PublisherClient<Channel>>,
    pub(crate) subscriber: Mutex<api::subscriber_client::SubscriberClient<Channel>>,
}

impl Client {
    pub(crate) const DOMAIN_NAME: &'static str = "pubsub.googleapis.com";
    pub(crate) const ENDPOINT: &'static str = "https://pubsub.googleapis.com";
    pub(crate) const SCOPES: [&'static str; 2] = [
        "https://www.googleapis.com/auth/cloud-platform",
        "https://www.googleapis.com/auth/pubsub",
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
        let tls_config = ClientTlsConfig::new()
            .ca_certificate(Certificate::from_pem(TLS_CERTS))
            .domain_name(Client::DOMAIN_NAME);

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
            .tls_config(tls_config)
            .connect()
            .await?;

        Ok(Client {
            project_name: project_name.into(),
            publisher: Mutex::new(api::publisher_client::PublisherClient::new(channel.clone())),
            subscriber: Mutex::new(api::subscriber_client::SubscriberClient::new(channel)),
        })
    }

    /// Create a new topic.
    pub async fn create_topic(
        &self,
        topic_name: &str,
        config: TopicConfig,
    ) -> Result<Topic<'_>, Error> {
        let mut service = self.publisher.lock().unwrap();

        let request = api::Topic {
            name: format!(
                "projects/{0}/topics/{1}",
                self.project_name.as_str(),
                topic_name
            ),
            labels: config.labels,
            message_storage_policy: None,
            kms_key_name: String::new(),
        };
        let response = service.create_topic(request).await?;
        let topic = response.into_inner();
        let name = topic.name.split('/').last().unwrap_or(topic_name);

        Ok(Topic::new(&self, name))
    }

    /// List all exisiting topics.
    pub async fn topics(&self) -> Result<Vec<Topic<'_>>, Error> {
        let mut topics = Vec::new();
        let page_size = 25;
        let mut page_token = String::default();
        let mut service = self.publisher.lock().unwrap();

        loop {
            let request = api::ListTopicsRequest {
                project: format!("projects/{0}", self.project_name.as_str()),
                page_size,
                page_token,
            };
            let response = service.list_topics(request).await?;
            let response = response.into_inner();
            page_token = response.next_page_token;
            topics.extend(response.topics.into_iter().map(|topic| {
                let name = topic.name.split('/').last().unwrap();
                Topic::new(&self, name)
            }));
            if page_token.is_empty() {
                break;
            }
        }

        Ok(topics)
    }

    /// Get a handle to a specific topic.
    pub async fn topic(&self, topic_name: &str) -> Result<Option<Topic<'_>>, Error> {
        let mut service = self.publisher.lock().unwrap();
        let request = api::GetTopicRequest {
            topic: format!(
                "projects/{0}/topics/{1}",
                self.project_name.as_str(),
                topic_name
            ),
        };
        let response = service.get_topic(request).await?;
        let topic = response.into_inner();
        let name = topic.name.split('/').last().unwrap_or(topic_name);

        Ok(Some(Topic::new(&self, name)))
    }

    /// List all existing subscriptions (to any topic).
    pub async fn subscriptions(&self) -> Result<Vec<Subscription<'_>>, Error> {
        let mut subscriptions = Vec::new();
        let page_size = 25;
        let mut page_token = String::default();
        let mut service = self.subscriber.lock().unwrap();

        loop {
            let request = api::ListSubscriptionsRequest {
                project: format!("projects/{0}", self.project_name.as_str()),
                page_size,
                page_token,
            };
            let response = service.list_subscriptions(request).await?;
            let response = response.into_inner();
            page_token = response.next_page_token;
            subscriptions.extend(response.subscriptions.into_iter().map(|subscription| {
                let name = subscription.name.split('/').last().unwrap();
                Subscription::new(&self, name)
            }));
            if page_token.is_empty() {
                break;
            }
        }

        Ok(subscriptions)
    }

    /// Get a handle of a specific subscription.
    pub async fn subscription(
        &self,
        subscription_name: &str,
    ) -> Result<Option<Subscription<'_>>, Error> {
        let mut service = self.subscriber.lock().unwrap();

        let request = api::GetSubscriptionRequest {
            subscription: format!(
                "projects/{0}/subscriptions/{1}",
                self.project_name.as_str(),
                subscription_name
            ),
        };
        let response = service.get_subscription(request).await?;
        let subscription = response.into_inner();
        let name = subscription.name.split('/').last();
        let name = name.unwrap_or(subscription_name);

        Ok(Some(Subscription::new(&self, name)))
    }
}
