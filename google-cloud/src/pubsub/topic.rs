use std::collections::HashMap;

use crate::pubsub::api;
use crate::pubsub::{Client, Error, Subscription, SubscriptionConfig};

/// Represents the topic's configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopicConfig {
    pub(crate) labels: HashMap<String, String>,
}

impl TopicConfig {
    /// Attach a label to the topic.
    pub fn label(mut self, name: impl Into<String>, value: impl Into<String>) -> TopicConfig {
        self.labels.insert(name.into(), value.into());
        self
    }
}

impl Default for TopicConfig {
    fn default() -> TopicConfig {
        TopicConfig {
            labels: HashMap::new(),
        }
    }
}

/// Represents a topic.
#[derive(Clone)]
pub struct Topic {
    pub(crate) client: Client,
    pub(crate) name: String,
}

impl Topic {
    pub(crate) fn new(client: Client, name: impl Into<String>) -> Topic {
        Topic {
            client,
            name: name.into(),
        }
    }

    /// Returns the unique identifier within its project
    pub fn id(&self) -> &str {
        self.name.rsplit('/').next().unwrap()
    }

    /// Create a subscription tied to this topic.
    pub async fn create_subscription(
        &mut self,
        id: &str,
        config: SubscriptionConfig,
    ) -> Result<Subscription, Error> {
        let request = api::Subscription {
            name: format!(
                "projects/{0}/subscriptions/{1}",
                self.client.project_name.as_str(),
                id,
            ),
            topic: self.name.clone(),
            ack_deadline_seconds: config.ack_deadline_duration.num_seconds() as i32,
            retain_acked_messages: config.message_retention_duration.is_some(),
            message_retention_duration: config.message_retention_duration.map(|mut dur| {
                let seconds = dur.num_seconds();
                dur = dur - chrono::Duration::seconds(seconds);
                let nanos = dur.num_nanoseconds().unwrap_or(0) as i32;
                prost_types::Duration { seconds, nanos }
            }),
            labels: config.labels,
            enable_message_ordering: false,
            push_config: None,
            expiration_policy: None,
            dead_letter_policy: None,
        };
        let request = self.client.construct_request(request).await?;
        let response = self.client.subscriber.create_subscription(request).await?;
        let subscription = response.into_inner();

        Ok(Subscription::new(self.client.clone(), subscription.name))
    }

    /// Publish a message onto this topic.
    pub async fn publish_message(&mut self, msg: api::PubsubMessage) -> Result<(), Error> {
        self.publish_messages(vec![msg]).await
    }

    /// Publish a message onto this topic.
    pub async fn publish_messages(
        &mut self,
        messages: Vec<api::PubsubMessage>,
    ) -> Result<(), Error> {
        let request = api::PublishRequest {
            topic: self.name.clone(),
            messages,
        };
        let request = self.client.construct_request(request).await?;
        self.client.publisher.publish(request).await?;

        Ok(())
    }

    /// Publish a message onto this topic.
    pub async fn publish(&mut self, data: impl Into<Vec<u8>>) -> Result<(), Error> {
        self.publish_message(api::PubsubMessage {
            data: data.into(),
            ..Default::default()
        })
        .await
    }

    /// Delete the topic.
    pub async fn delete(mut self) -> Result<(), Error> {
        let request = api::DeleteTopicRequest {
            topic: self.name.clone(),
        };
        let request = self.client.construct_request(request).await?;
        self.client.publisher.delete_topic(request).await?;

        Ok(())
    }
}
