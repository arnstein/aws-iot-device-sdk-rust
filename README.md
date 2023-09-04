[![Documentation](https://docs.rs/aws-iot-device-sdk-rust/badge.svg)](https://docs.rs/aws-iot-device-sdk-rust/)
[![crates.io](https://img.shields.io/crates/v/aws-iot-device-sdk-rust)](https://crates.io/crates/aws-iot-device-sdk-rust)


# aws-iot-device-sdk-rust

The AWS IoT Device SDK for Rust allows developers to write Rust to use their devices to access the AWS IoT platform through MQTT.
This is my first crate, and project, in Rust, and as I am still learning it will hopefully get a lot better with time.
I have recently done a revamp of this crate, as I hadn't had time to update it in a while.
Current functionality is:
- connect
- listen to incoming events
- subscribe to topic
- publish to topic



# Usage

1. Using the AWSIoTAsyncClient and event broadcast
Create an AWSIoTAsyncClient, then spawn a thread where it listens to incoming events with listen((eventloop, event_sender)). The incoming messages received in this thread will be broadcast to all the receivers. To acquire a new receiver, call client.get_receiver().
See examples folder for both sync and async examples.
Example:

```
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let aws_settings = client::AWSIoTSettings::new(
        "clientid".to_owned(),
        "AmazonRootCA1.pem".to_owned(),
        "cert.crt".to_owned(),
        "key.pem".to_owned(),
        "endpoint.amazonaws.com".to_owned(),
        None
        );

    let (iot_core_client, eventloop_stuff) = client::AWSIoTAsyncClient::new(aws_settings).await?;

    iot_core_client.subscribe("test".to_string(), QoS::AtMostOnce).await.unwrap();
    iot_core_client.publish("topic".to_string(), QoS::AtMostOnce, "hey").await.unwrap();

    let mut receiver1 = iot_core_client.get_receiver().await;
    let mut receiver2 = iot_core_client.get_receiver().await;

    let recv1_thread = tokio::spawn(async move {
        loop {
            match receiver1.recv().await {
                Ok(event) => {
                    match event {
                        Packet::Publish(p) => println!("Received message {:?} on topic: {}", p.payload, p.topic),
                        _ => println!("Got event on receiver1: {:?}", event),
                    }

                },
                Err(_) => (),
            }
        }
    });

    let recv2_thread = tokio::spawn(async move {
        loop {
            match receiver2.recv().await {
                Ok(event) => println!("Got event on receiver2: {:?}", event),
                Err(_) => (),
            }
        }
    });

    let listen_thread = tokio::spawn(async move {
            client::async_event_loop_listener(eventloop_stuff).await.unwrap();
    });

    tokio::join!(
        recv1_thread,
        recv2_thread,
        listen_thread);
    Ok(())
}
```

2. Get Rumqttc client and eventloop and implement it in your own way
If, for some reason, the current implementation does not work for your code, you could always just create a client and use the get_client() function to easily get a connection to IoT Core, and then implement the rest yourself.
Consult the rumqttc documentation to see how you can use the client and eventloop.

Example:

```
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let aws_settings = client::AWSIoTSettings::new(
        "clientid".to_owned(),
        "AmazonRootCA1.pem".to_owned(),
        "cert.crt".to_owned(),
        "key.pem".to_owned(),
        "endpoint.amazonaws.com".to_owned(),
        None
        );

    let (iot_core_client, (eventloop, _))  = client::AWSIoTAsyncClient::new(aws_settings).await?;
    let client = iot_core_client.get_client();
}
```
