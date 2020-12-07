use std::io::{self, Write};

use serde::{Deserialize, Serialize};

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
    assert_eq!(received.is_none(), true);
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
