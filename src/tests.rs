use std::io::{self, Write};

use json::json;

use crate::pubsub;

#[tokio::test]
async fn pubsub_connects_successfully() {
    let client = pubsub::Client::new(env!("GCP_TEST_PROJECT"));
    let topics = client.topics().await;
    assert!(topics.is_ok());
}

#[tokio::test]
async fn pubsub_sends_and_receives_message_successfully() {
    let client = pubsub::Client::new(env!("GCP_TEST_PROJECT"));

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
    let topic = topic.unwrap();
    println!("OK !");

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

    print!("serializing message... ");
    io::stdout().flush().unwrap();
    let data = json!({ "hello": "world !" });
    let message = json::to_vec(&data).unwrap();
    println!("OK !");

    print!("sending message... ");
    io::stdout().flush().unwrap();
    let result = topic.publish(message).await;
    assert!(result.is_ok());
    println!("OK !");

    print!("receiving message... ");
    io::stdout().flush().unwrap();
    let received = subscription.receive().await;
    assert!(received.is_some());
    let received = received.unwrap();
    println!("OK !");

    print!("acknowledging message... ");
    let result = received.ack().await;
    assert!(result.is_ok());
    println!("OK !");

    print!("deserializing message... ");
    io::stdout().flush().unwrap();
    let data = json::from_slice::<json::Value>(received.data());
    assert!(data.is_ok());
    println!("OK !");

    print!("deleting subscription... ");
    io::stdout().flush().unwrap();
    let result = subscription.delete().await;
    assert!(result.is_ok());
    println!("OK !");

    print!("deleting topic... ");
    io::stdout().flush().unwrap();
    let result = topic.delete().await;
    assert!(result.is_ok());
    println!("OK !");
}
