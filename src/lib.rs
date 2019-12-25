//! aws-iot-core-sdk-rust aims to be a well-functioning and easy to use AWS IoT device SDK.
//! At its core it uses the pure Rust MQTT client Rumqtt, as well as Serde for (de)serializing
//! JSON. The name is chosen to match its C, C++, Python and JS counterparts.
//! * Use this to easily connect your  IoT devices to AWS IoT Core.
//! * Publish and subscribe to any topic you want.
//! * Register callback functions that will be called to handle incoming messages on any topic.
//!
//! ## Publish and subscribe
//! ```no_run
//! use aws-iot-device-sdk-rust::AWSIoTClient;
//!
//! fn main() {
//!     let mut iot_core_client = client::AWSIoTClient::new(
//!         "myClientId",
//!         "root-CA.crt",
//!         "device.cert.pem",
//!         "device.private.key",
//!         "myendpoint.iot.eu-west-1.amazonaws.com"
//!         );
//!
//!     iot_core_client.subscribe("thing/light/status", QoS::AtLeastOnce);
//!     iot_core_client.publish("thing/light/status", "on");
//! }
//!```

//! ## Add callback
//! ```no_run
//! use aws-iot-device-sdk-rust::AWSIoTClient;
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
//!         );
//!
//!     iot_core_client.add_callback("thing/light/status", my_callback);
//! }
//!```

pub mod client;
pub mod shadow;

pub use crate::client::AWSIoTClient;
pub use crate::shadow::AWSShadowManager;
