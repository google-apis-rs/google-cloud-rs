use std::io::{self, Write};

use futures::sink::SinkExt;
use futures::stream::TryStreamExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task::spawn;
use tokio::time::{sleep, Duration};

use crate::pubsub;

macro_rules! assert_ok {
    ($expr:expr) => {
        match $expr {
            Ok(value) => value,
            Err(err) => {
                panic!("asserted result is an error: {}", err);
            }
        }
    };
}

macro_rules! assert_some {
    ($expr:expr) => {
        match $expr {
            Some(value) => value,
            None => {
                panic!("asserted option is an none");
            }
        }
    };
}

async fn setup_client() -> Result<pubsub::Client, pubsub::Error> {
    let creds = super::load_creds();
    pubsub::Client::from_credentials(env!("GCP_TEST_PROJECT"), creds).await
}

#[tokio::test]
async fn pubsub_lists_topics() {
    //? Setup test client.
    let mut client = assert_ok!(setup_client().await);

    //? List all topics of the project.
    assert_ok!(client.topics().await);
}

#[tokio::test]
async fn pubsub_sends_and_receives_message_successfully() {
    //? Setup test client.
    let mut client = assert_ok!(setup_client().await);

    //? Acquire topic, or create it if needed.
    print!("acquiring topic... ");
    io::stdout().flush().unwrap();
    let config = pubsub::TopicConfig::default();
    let topic = match client.create_topic(env!("GCP_TEST_TOPIC"), config).await {
        Ok(topic) => Ok(Some(topic)),
        Err(_) => client.topic(env!("GCP_TEST_TOPIC")).await,
    };
    let mut topic = assert_some!(assert_ok!(topic));
    println!("OK !");

    //? Acquire subscription, or create it if needed.
    print!("acquiring subscription... ");
    io::stdout().flush().unwrap();
    let config = pubsub::SubscriptionConfig::default();
    let subscription = match topic
        .create_subscription(env!("GCP_TEST_SUBSCRIPTION"), config)
        .await
    {
        Ok(subscription) => Ok(Some(subscription)),
        Err(_) => client.subscription(env!("GCP_TEST_SUBSCRIPTION")).await,
    };
    let mut subscription = assert_some!(assert_ok!(subscription));
    println!("OK !");

    //? Prepare message.
    print!("serializing message... ");
    io::stdout().flush().unwrap();
    #[derive(Serialize, Deserialize)]
    struct Message<'a> {
        name: &'a str,
        value: &'a str,
    }
    let data = Message {
        name: "hello",
        value: "world !",
    };
    let message = assert_ok!(json::to_vec(&data));
    println!("OK !");

    //? Publish that message onto the topic.
    print!("sending message... ");
    io::stdout().flush().unwrap();
    assert_ok!(topic.publish(message).await);
    println!("OK !");

    //? Receive it back from the subscription.
    print!("receiving message... ");
    io::stdout().flush().unwrap();
    let mut received = assert_some!(subscription.receive().await);
    println!("OK !");

    //? Acknowledge the reception of that message.
    print!("acknowledging message... ");
    io::stdout().flush().unwrap();
    assert_ok!(received.ack().await);
    println!("OK !");

    //? Deserialize the message data.
    print!("deserializing message... ");
    io::stdout().flush().unwrap();
    assert_ok!(json::from_slice::<Message>(received.data()));
    println!("OK !");

    let received = subscription
        .receive_with_options(pubsub::ReceiveOptions {
            return_immediately: true,
            max_messages: 1,
        })
        .await;
    assert!(received.is_none());
    println!("OK !");

    //? Delete the subscription.
    print!("deleting subscription... ");
    io::stdout().flush().unwrap();
    assert_ok!(subscription.delete().await);
    println!("OK !");

    //? Delete the topic.
    print!("deleting topic... ");
    io::stdout().flush().unwrap();
    assert_ok!(topic.delete().await);
    println!("OK !");
}

/// This example demonstrates using the `StreamingPull` API in order to pull messages
/// efficiently from the pubsub endpoint, process them in parallel, and acknowledge them on
/// the stream. It also handles errors by re-establishing the stream.
#[tokio::test]
async fn pubsub_sends_and_receives_stream_successfully() {
    //? Setup test client.
    let mut client = assert_ok!(setup_client().await);

    //? Acquire topic, or create it if needed.
    print!("acquiring topic... ");
    io::stdout().flush().unwrap();
    let config = pubsub::TopicConfig::default();
    let topic = match client.create_topic(env!("GCP_TEST_TOPIC"), config).await {
        Ok(topic) => Ok(Some(topic)),
        Err(_) => client.topic(env!("GCP_TEST_TOPIC")).await,
    };
    let mut topic = assert_some!(assert_ok!(topic));
    println!("OK !");

    //? Acquire subscription, or create it if needed.
    print!("acquiring subscription... ");
    io::stdout().flush().unwrap();
    let config = pubsub::SubscriptionConfig::default();
    let subscription = match topic
        .create_subscription(env!("GCP_TEST_SUBSCRIPTION"), config)
        .await
    {
        Ok(subscription) => Ok(Some(subscription)),
        Err(_) => client.subscription(env!("GCP_TEST_SUBSCRIPTION")).await,
    };
    let mut subscription = assert_some!(assert_ok!(subscription));
    println!("OK !");

    //? Prepare message.
    print!("serializing message... ");
    io::stdout().flush().unwrap();
    #[derive(Serialize, Deserialize)]
    struct Message<'a> {
        name: &'a str,
        value: &'a str,
    }
    let msgs = [
        assert_ok!(json::to_vec(&Message {
            name: "hello",
            value: "1",
        })),
        assert_ok!(json::to_vec(&Message {
            name: "hello",
            value: "2",
        })),
        assert_ok!(json::to_vec(&Message {
            name: "hello",
            value: "3",
        })),
        assert_ok!(json::to_vec(&Message {
            name: "hello",
            value: "4",
        })),
        assert_ok!(json::to_vec(&Message {
            name: "hello",
            value: "5",
        })),
    ];
    println!("OK !");

    //? Publish messages onto the topic.
    print!("sending messages... ");
    io::stdout().flush().unwrap();
    for msg in msgs {
        assert_ok!(topic.publish(msg).await);
    }
    println!("OK !");

    println!("creating streams...");
    let receive_opts = pubsub::ReceiveStreamOptions {
        ack_ids: Vec::new(),
        modify_deadline_ack_ids: Vec::new(),
        modify_deadline_seconds: Vec::new(),
        stream_ack_deadline_seconds: 10,
    };
    let (mut recv, mut send) = assert_ok!(subscription.pull_streaming(receive_opts).await);
    println!("OK !");

    // Use a semaphore to ensure that we don't process too many messages at once.
    const MAX_PARALLEL_PROC: usize = 3;
    let sem = Arc::new(Semaphore::new(MAX_PARALLEL_PROC));

    // Loop until we've received all messages.
    let mut received = 0;

    while received < 5 {
        println!("checking for next message(s)...");
        match recv.try_next().await {
            Ok(msgs) => {
                if let Some(msgs) = msgs {
                    println!("got {} messages...", msgs.len());
                    received += msgs.len();
                    for msg in msgs {
                        let permit = Arc::clone(&sem).acquire_owned().await;
                        let mut send = send.clone();
                        spawn(async move {
                            let _permit = permit;

                            //? Acknowledge the reception of that message.
                            // You should generally do this only after you've handled the
                            // message in some way, to ensure that it doesn't get lost.
                            print!("acknowledging message... ");
                            let receive_opts = pubsub::ReceiveStreamOptions {
                                ack_ids: vec![msg.ack_id().to_string()],
                                modify_deadline_ack_ids: Vec::new(),
                                modify_deadline_seconds: Vec::new(),
                                stream_ack_deadline_seconds: 10,
                            };
                            assert_ok!(send.send(receive_opts).await);
                            println!("OK !");

                            //? Deserialize the message data.
                            print!("deserializing message... ");
                            io::stdout().flush().unwrap();
                            assert_ok!(json::from_slice::<Message>(msg.data()));
                            println!("OK !");
                        });
                    }
                } else {
                    // This doesn't ever seem to happen, but let's handle it just in case.
                    println!("Lost subscription stream, trying again in 5s");
                    sleep(Duration::from_secs(5)).await;
                    let rs = assert_ok!(subscription.pull_streaming(receive_opts.clone()).await);
                    recv = rs.0;
                    send = rs.1;
                }
            }
            Err(e) => {
                // TODO: check error type to make sure we're handling the error correctly
                // (e.g. error "8a75" is retryable, but others might not be). For now,
                // always retry.
                println!("Subscription stream error {:?}, trying again in 5s", e);
                sleep(Duration::from_secs(5)).await;
                let rs = assert_ok!(subscription.pull_streaming(receive_opts.clone()).await);
                recv = rs.0;
                send = rs.1;
            }
        }
    }

    println!("Verifying no messages left.");
    let received = subscription
        .receive_with_options(pubsub::ReceiveOptions {
            return_immediately: true,
            max_messages: 1,
        })
        .await;
    assert!(received.is_none());
    println!("OK !");

    //? Delete the subscription.
    print!("deleting subscription... ");
    io::stdout().flush().unwrap();
    assert_ok!(subscription.delete().await);
    println!("OK !");

    //? Delete the topic.
    print!("deleting topic... ");
    io::stdout().flush().unwrap();
    assert_ok!(topic.delete().await);
    println!("OK !");
}
