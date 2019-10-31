use std::collections::HashMap;

use crate::pubsub::{api, Client};

/// Represents a received message (from a subscription).
pub struct Message<'a> {
    pub(crate) client: &'a Client,
    pub(crate) data: Vec<u8>,
    pub(crate) attributes: HashMap<String, String>,
    pub(crate) ack_id: String,
    pub(crate) message_id: String,
    pub(crate) publish_time: chrono::NaiveDateTime,
    pub(crate) subscription_name: String,
}

impl<'a> Message<'a> {
    /// The message's unique ID.
    pub fn id(&self) -> &str {
        self.message_id.as_str()
    }

    /// The payload data of the message.
    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    /// The attributes of the message.
    pub fn attributes(&self) -> &HashMap<String, String> {
        &self.attributes
    }

    /// The publication time of the message.
    pub fn publish_time(&self) -> chrono::NaiveDateTime {
        self.publish_time
    }

    /// Indicate that this client processed or will process the message successfully.
    ///
    /// If a message isn't acknowledged, it will be redelivered to other subscribers.
    pub async fn ack(&self) -> Result<(), Box<dyn std::error::Error + 'static>> {
        let mut service = self.client.subscriber.lock().unwrap();

        let request = api::AcknowledgeRequest {
            subscription: format!(
                "projects/{0}/subscriptions/{1}",
                self.client.project_name.as_str(),
                self.subscription_name,
            ),
            ack_ids: vec![self.ack_id.clone()],
        };
        service.acknowledge(request).await?;

        Ok(())
    }

    /// Indicate that this client won't process the message.
    ///
    /// This allows Pub/Sub to redeliver the message more quickly than by awaiting the acknowledgement timeout.
    pub async fn nack(&self) -> Result<(), Box<dyn std::error::Error + 'static>> {
        let mut service = self.client.subscriber.lock().unwrap();

        let request = api::ModifyAckDeadlineRequest {
            subscription: format!(
                "projects/{0}/subscriptions/{1}",
                self.client.project_name.as_str(),
                self.subscription_name,
            ),
            ack_ids: vec![self.ack_id.clone()],
            ack_deadline_seconds: 0,
        };
        service.modify_ack_deadline(request).await?;

        Ok(())
    }
}
