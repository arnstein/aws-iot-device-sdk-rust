//! aws-iot-core-sdk-rust aims to be a well-functioning and easy to use AWS IoT device SDK.
//! At its core it uses the pure Rust MQTT client rumqttc. The name is chosen to match its C, C++, Python and JS counterparts.
//! * Use this to easily connect your IoT devices to AWS IoT Core.
//! * Publish and subscribe to any topic you want.
//! * Implement the AWSEventHandler trait for your struct.
//!
//! The crate re-exports Mqtt311s Quality of Service enum. These are used when subscribing and
//! publish. The variants are:
//! * AtMostOnce (0)
//! * AtLeastOnce (1)
//! * ExactlyOnce (2)
//!
//! ## Publish and subscribe
//! ```no_run
//!#[tokio::main]
//!async fn main() -> Result<(), Box<dyn Error>> {
//!    let aws_settings = client::AWSIoTSettings::new(
//!        "clientid".to_owned(),
//!        "AmazonRootCA1.pem".to_owned(),
//!        "cert.crt".to_owned(),
//!        "key.pem".to_owned(),
//!        "endpoint.amazonaws.com".to_owned(),
//!        None
//!        );
//!
//!    let (iot_core_client, eventloop_stuff) = client::AWSIoTAsyncClient::new(aws_settings).await?;
//!
//!    iot_core_client.subscribe("test".to_string(), QoS::AtMostOnce).await.unwrap();
//!    iot_core_client.publish("topic".to_string(), QoS::AtMostOnce, "hey").await.unwrap();
//!
//!    let mut receiver1 = iot_core_client.get_receiver().await;
//!    let mut receiver2 = iot_core_client.get_receiver().await;
//!
//!    let recv1_thread = tokio::spawn(async move {
//!        loop {
//!            match receiver1.recv().await {
//!                Ok(event) => {
//!                    match event {
//!                        Packet::Publish(p) => println!("Received message {:?} on topic: {}", p.payload, p.topic),
//!                        _ => println!("Got event on receiver1: {:?}", event),
//!                    }
//!
//!                },
//!                Err(_) => (),
//!            }
//!        }
//!    });
//!
//!    let recv2_thread = tokio::spawn(async move {
//!        loop {
//!            match receiver2.recv().await {
//!                Ok(event) => println!("Got event on receiver2: {:?}", event),
//!                Err(_) => (),
//!            }
//!        }
//!    });
//!
//!    let listen_thread = tokio::spawn(async move {
//!            client::async_event_loop_listener(eventloop_stuff).await.unwrap();
//!    });
//!
//!    tokio::join!(
//!        recv1_thread,
//!        recv2_thread,
//!        listen_thread);
//!    Ok(())
//!}
//!
//!```

pub mod client;
pub mod error;

pub use serde_json::json;
pub use rumqttc::{QoS, Packet, Publish, EventLoop};
pub use self::client::{AWSIoTAsyncClient, AWSIoTSettings, async_event_loop_listener};
pub use self::error::AWSIoTError;
