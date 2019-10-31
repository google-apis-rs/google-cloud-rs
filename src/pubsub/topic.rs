use std::collections::HashMap;

use crate::pubsub::{api, Client, Subscription, SubscriptionConfig};

/// Represents the topic's configuration.
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
    /// Get the default topic configuration.
    fn default() -> TopicConfig {
        TopicConfig {
            labels: HashMap::new(),
        }
    }
}

/// Represents a topic.
pub struct Topic<'a> {
    pub(crate) client: &'a Client,
    pub(crate) name: String,
}

impl<'a> Topic<'a> {
    pub(crate) fn new(client: &'a Client, name: impl Into<String>) -> Topic<'a> {
        Topic {
            client,
            name: name.into(),
        }
    }

    /// Create a subscription tied to this topic.
    pub async fn create_subscription(
        &self,
        name: &str,
        config: SubscriptionConfig,
    ) -> Result<Subscription<'a>, Box<dyn std::error::Error + 'static>> {
        let mut service = self.client.subscriber.lock().unwrap();

        let request = api::Subscription {
            name: format!(
                "projects/{0}/subscriptions/{1}",
                self.client.project_name.as_str(),
                name,
            ),
            topic: format!(
                "projects/{0}/topics/{1}",
                self.client.project_name.as_str(),
                self.name,
            ),
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
        let response = service.create_subscription(request).await?;
        let subscription = response.into_inner();
        let name = subscription.name.split('/').last().unwrap_or(name);

        Ok(Subscription::new(self.client, name))
    }

    /// Publish a message onto this topic.
    pub async fn publish(
        &self,
        data: impl Into<Vec<u8>>,
    ) -> Result<(), Box<dyn std::error::Error + 'static>> {
        let mut service = self.client.publisher.lock().unwrap();

        let request = api::PublishRequest {
            topic: format!(
                "projects/{0}/topics/{1}",
                self.client.project_name.as_str(),
                self.name,
            ),
            messages: vec![api::PubsubMessage {
                data: data.into(),
                attributes: HashMap::new(),
                message_id: String::new(),
                ordering_key: String::new(),
                publish_time: None,
            }],
        };
        service.publish(request).await?;
        // let response = response.into_inner();

        Ok(())
    }

    /// Delete the topic.
    pub async fn delete(self) -> Result<(), Box<dyn std::error::Error + 'static>> {
        let mut service = self.client.publisher.lock().unwrap();

        let request = api::DeleteTopicRequest {
            topic: format!(
                "projects/{0}/topics/{1}",
                self.client.project_name.as_str(),
                self.name,
            ),
        };
        service.delete_topic(request).await?;

        Ok(())
    }
}
