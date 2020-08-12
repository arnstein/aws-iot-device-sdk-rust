//! aws-iot-core-sdk-rust aims to be a well-functioning and easy to use AWS IoT device SDK.
//! At its core it uses the pure Rust MQTT client Rumqtt, as well as Serde for (de)serializing
//! JSON. The name is chosen to match its C, C++, Python and JS counterparts.
//! * Use this to easily connect your  IoT devices to AWS IoT Core.
//! * Publish and subscribe to any topic you want.
//! * Register callback functions that will be called to handle incoming messages on any topic.
//!
//! The crate re-exports Mqtt311s Quality of Service enum. These are used when subscribing and
//! publish. The variants are:
//! * AtMostOnce (0)
//! * AtLeastOnce (1)
//! * ExactlyOnce (2)
//!
//! ## Publish and subscribe
//! ```no_run
//! use aws_iot_device_sdk_rust::client;
//!
//! fn main() {
//!     let mut iot_core_client = client::AWSIoTClient::new(
//!         "myClientId",
//!         "root-CA.crt",
//!         "device.cert.pem",
//!         "device.private.key",
//!         "myendpoint.iot.eu-west-1.amazonaws.com"
//!         ).unwrap();
//!
//!     iot_core_client.start_listening();
//!     iot_core_client.subscribe("thing/light/status", QoS::AtLeastOnce);
//!     iot_core_client.publish("thing/light/status", "on");
//! }
//!```
//!
//! ## Add callback
//! ```no_run
//! use aws_iot_device_sdk_rust::client;
//!
//! fn my_callback() {
//!     println!("Someone or something published to thing/light/status!");
//! }
//! fn main() {
//!     let mut iot_core_client = client::AWSIoTClient::new(
//!         "myClientId",
//!         "root-CA.crt",
//!         "device.cert.pem",
//!         "device.private.key",
//!         "myendpoint.iot.eu-west-1.amazonaws.com"
//!         ).unwrap();
//!
//!     iot_core_client.start_listening();
//!     iot_core_client.add_callback("thing/light/status", my_callback).unwrap();
//! }
//!```
//!
//! ## Get shadow updates
//! ```no_run
//! use aws_iot_device_sdk_rust::{client, shadow};
//!
//! fn print_shadow_updates(shadow: String) {
//!     println!("{:?}", shadow);
//! }
//!
//! fn main() {
//!     let mut iot_core_client = client::AWSIoTClient::new(
//!         "myClientId",
//!         "root-CA.crt",
//!         "device.cert.pem",
//!         "device.private.key",
//!         "myendpoint.iot.eu-west-1.amazonaws.com"
//!         ).unwrap();
//!
//!     let mut shadow_manager = shadow::AWSShadowManager::new(&mut iot_core_client,
//!         String::from("MyThing"));
//!     shadow_manager.add_listen_on_delta_callback(print_shadow_updates);
//! }
//!```

pub mod client;

pub use crate::client::AWSIoTSyncClient;
pub use crate::client::AWSIoTAsyncClient;
pub use serde_json::json;
pub use rumqttc::QoS;
