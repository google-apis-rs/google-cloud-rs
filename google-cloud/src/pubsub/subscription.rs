use crate::pubsub::api::ReceivedMessage;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use chrono::{Duration, NaiveDateTime};
use futures::channel::mpsc::Sender;
use futures::lock::Mutex;
use futures::stream::{Stream, StreamExt};
use futures::FutureExt;
use futures::SinkExt;
use tonic::{Status, Streaming};

use crate::pubsub::api;
use crate::pubsub::{Client, Error, Message};

/// Represents the subscription's configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscriptionConfig {
    pub(crate) ack_deadline_duration: Duration,
    pub(crate) message_retention_duration: Option<Duration>,
    pub(crate) labels: HashMap<String, String>,
}

impl SubscriptionConfig {
    /// Set the message acknowledgement duration.
    pub fn ack_deadline(mut self, duration: Duration) -> SubscriptionConfig {
        self.ack_deadline_duration = duration;
        self
    }

    /// Enable message retention and set its duration.
    pub fn retain_messages(mut self, duration: Duration) -> SubscriptionConfig {
        self.message_retention_duration = Some(duration);
        self
    }

    /// Attach a label to the subscription.
    pub fn label(
        mut self,
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> SubscriptionConfig {
        self.labels.insert(name.into(), value.into());
        self
    }
}

impl Default for SubscriptionConfig {
    fn default() -> SubscriptionConfig {
        SubscriptionConfig {
            ack_deadline_duration: Duration::seconds(10),
            message_retention_duration: None,
            labels: HashMap::new(),
        }
    }
}

/// Optional parameters for pull.
#[derive(Debug, Clone, PartialEq)]
pub struct ReceiveOptions {
    /// return immediately if there are no messages in the subscription
    pub return_immediately: bool,
    /// Number of messages to retrieve at once
    pub max_messages: i32,
}

impl Default for ReceiveOptions {
    fn default() -> Self {
        Self {
            return_immediately: false,
            max_messages: 1,
        }
    }
}

/// Optional parameters for streaming pull
#[derive(Debug, Clone, PartialEq)]
pub struct StreamingOptions {
    /// Client ID: identifies the client in the Google Cloud, state is shared between clients with
    /// the same ID.
    pub client_id: String,
    /// Maximum number of un-ACK'd messages sent through the stream at any given moment. Can be
    /// used to limit memory usage at the cost of throughput.
    pub max_messages: i64,
    /// ACK deadline for this stream.
    pub ack_deadline: i32,
    /// Filter messages resent to the subscription.
    pub filter_redeliveries: bool,
}

impl Default for StreamingOptions {
    fn default() -> Self {
        Self {
            client_id: "google-cloud-rs".into(),
            max_messages: 0,
            ack_deadline: 10,
            filter_redeliveries: false,
        }
    }
}

/// Represents a subscription, tied to a topic.
#[derive(Clone)]
pub struct Subscription {
    pub(crate) client: Client,
    pub(crate) name: String,
    pub(crate) buffer: VecDeque<api::ReceivedMessage>,
}

impl Subscription {
    pub(crate) fn new(client: Client, name: impl Into<String>) -> Subscription {
        Subscription {
            client,
            name: name.into(),
            buffer: VecDeque::new(),
        }
    }

    /// Returns the unique identifier within its project
    pub fn id(&self) -> &str {
        self.name.rsplit('/').next().unwrap()
    }

    /// Receive the next message from the subscription.
    pub async fn receive(&mut self) -> Result<Option<Message>, Error> {
        self.receive_with_options(Default::default()).await
    }

    /// Receive the next message from the subscription with options.
    pub async fn receive_with_options(
        &mut self,
        opts: ReceiveOptions,
    ) -> Result<Option<Message>, Error> {
        loop {
            if let Some(handle) = self.buffer.pop_front() {
                let message = handle.message.unwrap();
                let timestamp = message.publish_time.unwrap();
                let message = Message {
                    client: self.client.clone(),
                    subscription_name: self.name.clone(),
                    data: message.data,
                    message_id: message.message_id,
                    ack_id: handle.ack_id,
                    attributes: message.attributes,
                    publish_time: chrono::NaiveDateTime::from_timestamp(
                        timestamp.seconds,
                        timestamp.nanos as u32,
                    ),
                };
                break Ok(Some(message));
            } else {
                match self.pull(&opts).await {
                    Ok(messages) => {
                        if messages.is_empty() && opts.return_immediately {
                            break Ok(None);
                        } else {
                            self.buffer.extend(messages);
                        }
                    }
                    Err(_) => {}
                }
            }
        }
    }

    /// Start a stream of incoming messages.
    pub async fn stream(&mut self) -> Result<impl Stream<Item = Result<Message, Status>>, Error> {
        self.stream_with_options(Default::default()).await
    }

    /// Start a stream of incoming messages with options.
    pub async fn stream_with_options(
        &mut self,
        opts: StreamingOptions,
    ) -> Result<impl Stream<Item = Result<Message, Status>>, Error> {
        let filter_redeliveries = opts.filter_redeliveries;
        let (streaming, tx) = self.pull_streaming(opts).await?;
        let client = self.client.clone();
        let name = self.name.clone();
        let tx = Arc::new(Mutex::new(tx));
        Ok(futures::stream::unfold(streaming, |mut res| async {
            match res.message().await {
                Ok(Some(v)) => Some((Ok(v), res)),
                Ok(None) => None,
                // TODO: Better error handling?
                Err(err) => Some((Err(err), res)),
            }
        })
        .then({
            let tx = tx.clone();
            move |res| {
                let tx = tx.clone();
                async move {
                    match res {
                        Ok(v) => {
                            // TODO: Better end-user message acknowledgement mechanism
                            let mut tx = tx.lock().await;
                            Ok(tx
                                .send(
                                    v.received_messages
                                        .iter()
                                        .map(|m| m.ack_id.clone())
                                        .collect(),
                                )
                                .map(|res| {
                                    res.map(|()| v.received_messages).expect("Received closed")
                                })
                                .await)
                        }
                        Err(err) => Err(err),
                    }
                }
            }
        })
        .flat_map(|v: Result<Vec<ReceivedMessage>, Status>| match v {
            Ok(v) => futures::stream::iter(v).map(Ok).boxed(),
            Err(err) => futures::stream::once(futures::future::ready(Err(err))).boxed(),
        })
        .filter(move |m| {
            futures::future::ready(if filter_redeliveries {
                m.as_ref().map(|m| m.delivery_attempt == 0).unwrap_or(true) // Propagate errors through
            } else {
                true
            })
        })
        .filter(|m| futures::future::ready(m.as_ref().map(|m| m.message.is_some()).unwrap_or(true))) // Propagate errors through
        .map(move |m: Result<ReceivedMessage, Status>| {
            m.map(|m| {
                let inner_msg = m.message.unwrap();
                let raw_publish_time = inner_msg.publish_time.unwrap_or_default();
                Message {
                    client: client.clone(),
                    subscription_name: name.clone(),
                    ack_id: m.ack_id,
                    message_id: inner_msg.message_id,
                    publish_time: NaiveDateTime::from_timestamp(
                        raw_publish_time.seconds,
                        raw_publish_time.nanos as u32,
                    ),
                    attributes: inner_msg.attributes,
                    data: inner_msg.data,
                }
            })
        }))
    }

    /// Delete the subscription.
    pub async fn delete(mut self) -> Result<(), Error> {
        let request = api::DeleteSubscriptionRequest {
            subscription: self.name.clone(),
        };
        let request = self.client.construct_request(request).await?;
        self.client.subscriber.delete_subscription(request).await?;

        Ok(())
    }

    pub(crate) async fn pull(
        &mut self,
        opts: &ReceiveOptions,
    ) -> Result<Vec<api::ReceivedMessage>, Error> {
        let request = api::PullRequest {
            subscription: self.name.clone(),
            return_immediately: opts.return_immediately,
            max_messages: opts.max_messages,
        };
        let request = self.client.construct_request(request).await?;
        let response = self.client.subscriber.pull(request).await?;
        let response = response.into_inner();

        Ok(response.received_messages)
    }

    pub(crate) async fn pull_streaming(
        &mut self,
        opts: StreamingOptions,
    ) -> Result<(Streaming<api::StreamingPullResponse>, Sender<Vec<String>>), Error> {
        let request = api::StreamingPullRequest {
            subscription: self.name.clone(),
            client_id: opts.client_id,
            stream_ack_deadline_seconds: opts.ack_deadline,
            ..Default::default()
        };
        let (tx, rx) = futures::channel::mpsc::channel(1000);
        let request = self
            .client
            .construct_request(
                futures::stream::once(futures::future::ready(request)).chain(rx.map(move |ids| {
                    api::StreamingPullRequest {
                        ack_ids: ids,
                        ..Default::default()
                    }
                })),
            )
            .await?;
        let response = self.client.subscriber.streaming_pull(request).await?;
        Ok((response.into_inner(), tx))
    }
}

// impl<'a> Stream for Subscription<'a> {
//     type Item = Message<'a>;
//     fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
//         let fut = match self.fut {
//             Some(fut) => fut.as_mut(),
//             None => {
//                 self.fut.replace(Box::pin(self.next_message()));
//                 self.fut.as_mut().unwrap().as_mut()
//             }
//         };

//         fut.poll(cx)
//     }
// }
