[![Documentation](https://docs.rs/aws-iot-device-sdk-rust/badge.svg)](https://docs.rs/aws-iot-device-sdk-rust/)
[![crates.io](https://img.shields.io/crates/v/aws-iot-device-sdk-rust)](https://crates.io/crates/aws-iot-device-sdk-rust)

Functionality: 
- connect
- subscribe to topic
- listen to topic
- publish to topic

TODO:
message: 
type: shadow or publish,
payload: T

serialize message


# aws-iot-device-sdk-rust

The AWS IoT Device SDK for Rust allows developers to write Rust to use their devices to access the AWS IoT platform through MQTT.
This is my first crate, and project, in Rust, and as I am still learning it will hopefully get a lot better with time.
With the client you can publish and subscribe to topics and add callbacks that are associated with topics.
The shadow manager updates, gets, publishes and deletes the device shadow.

It has been through the Works on My Machine Certification Program, and it Works on My Machine™.


# Usage

There's several ways of using this crate.

1. Using the AWSIoTAsyncClient message queues
Create an AWSIoTAsyncClient, then spawn a thread where it listens to incoming events with client.listen(). The incoming messages received in this thread will be broadcast to all the receivers. To acquire a new receiver, call client.get_receiver().
Example:

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

    let mut iot_core_client = client::AWSIoTAsyncClient::new(aws_settings).await?;

    iot_core_client.subscribe("test".to_string(), QoS::AtMostOnce).await.unwrap();
    iot_core_client.publish("topic".to_string(), QoS::AtMostOnce, "hey").await.unwrap();

    let mut receiver1 = iot_core_client.get_receiver().await;
    let mut receiver2 = iot_core_client.get_receiver().await;

    let recv1_thread = tokio::spawn(async move {
        loop {
            match receiver1.recv().await {
                Ok(message) => println!("Got message on receiver1: {:?}", message),
                Err(_) => (),
            }
        }
    });

    let recv2_thread = tokio::spawn(async move {
        loop {
            match receiver2.recv().await {
                Ok(message) => println!("Got message on receiver2: {:?}", message),
                Err(_) => (),
            }
        }
    });
    let listen_thread = tokio::spawn(async move {
        loop {
            iot_core_client.listen().await.unwrap();
        }
    });

    let result = tokio::join!(
        recv1_thread,
        recv2_thread,
        listen_thread);

    Ok(())
}

2. Implementing the AWSEventHandler trait for your code and using the incoming_event_handler
If you want a callback based approach, implement the AWSEventHandler trait functions you can define what will happen on each incoming or outgoing message.
Example:

3. As an easy way to get your device connected to AWS IoT Core
The AWSIoTAsyncClient return the rumqttc client and eventloop, which you can use in your own code in any way you want (or at least in any way the borrow checker allows you to...).  
Consult the rumqttc documentation to see how you can use the client and eventloop.

Example:
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

    let mut iot_core_client = client::AWSIoTAsyncClient::new(aws_settings).await?;
    let (client, eventloop) = iot_core_client.get_client_and_eventloop();
}
