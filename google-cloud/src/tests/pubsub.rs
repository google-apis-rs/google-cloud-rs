use std::io::{self, Write};

use serde::{Deserialize, Serialize};

use crate::authorize::ApplicationCredentials;
use crate::pubsub;

async fn setup_client() -> Result<pubsub::Client, pubsub::Error> {
    let creds = json::from_str::<ApplicationCredentials>(env!("GCP_TEST_CREDENTIALS"))?;
    pubsub::Client::from_credentials(env!("GCP_TEST_PROJECT"), creds).await
}

#[tokio::test]
async fn pubsub_lists_topics() {
    //? Setup test client.
    let client = setup_client().await;
    assert!(client.is_ok());
    let mut client = client.unwrap();

    //? List all topics of the project.
    let topics = client.topics().await;
    assert!(topics.is_ok());
}

#[tokio::test]
async fn pubsub_sends_and_receives_message_successfully() {
    //? Setup test client.
    let client = setup_client().await;
    assert!(client.is_ok());
    let mut client = client.unwrap();

    //? Acquire topic, or create it if needed.
    print!("acquiring topic... ");
    io::stdout().flush().unwrap();
    let config = pubsub::TopicConfig::default();
    let topic = match client.create_topic(env!("GCP_TEST_TOPIC"), config).await {
        Ok(topic) => Ok(Some(topic)),
        Err(_) => client.topic(env!("GCP_TEST_TOPIC")).await,
    };
    assert!(topic.is_ok());
    let topic = topic.unwrap();
    assert!(topic.is_some());
    let mut topic = topic.unwrap();
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
    assert!(subscription.is_ok());
    let subscription = subscription.unwrap();
    assert!(subscription.is_some());
    let mut subscription = subscription.unwrap();
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
    let message = json::to_vec(&data).unwrap();
    println!("OK !");

    //? Publish that message onto the topic.
    print!("sending message... ");
    io::stdout().flush().unwrap();
    let result = topic.publish(message).await;
    assert!(result.is_ok());
    println!("OK !");

    //? Receive it back from the subscription.
    print!("receiving message... ");
    io::stdout().flush().unwrap();
    let received = subscription.receive().await;
    assert!(received.is_some());
    let mut received = received.unwrap();
    println!("OK !");

    //? Acknowledge the reception of that message.
    print!("acknowledging message... ");
    io::stdout().flush().unwrap();
    let result = received.ack().await;
    assert!(result.is_ok());
    println!("OK !");

    //? Deserialize the message data.
    print!("deserializing message... ");
    io::stdout().flush().unwrap();
    let data = json::from_slice::<Message>(received.data());
    assert!(data.is_ok());
    println!("OK !");

    //? Delete the subscription.
    print!("deleting subscription... ");
    io::stdout().flush().unwrap();
    let result = subscription.delete().await;
    assert!(result.is_ok());
    println!("OK !");

    //? Delete the topic.
    print!("deleting topic... ");
    io::stdout().flush().unwrap();
    let result = topic.delete().await;
    assert!(result.is_ok());
    println!("OK !");
}
