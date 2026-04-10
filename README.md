[![Documentation](https://docs.rs/aws-iot-device-sdk-rust/badge.svg)](https://docs.rs/aws-iot-device-sdk-rust/)
[![crates.io](https://img.shields.io/crates/v/aws-iot-device-sdk-rust)](https://crates.io/crates/aws-iot-device-sdk-rust)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

# aws-iot-device-sdk-rust

A Rust SDK for connecting IoT devices to [AWS IoT Core](https://aws.amazon.com/iot-core/) via MQTT 3.1.1. Built on top of [rumqttc](https://crates.io/crates/rumqttc) with TLS powered by [rustls](https://crates.io/crates/rustls).

## Features

- Async and sync clients for connecting to AWS IoT Core
- Mutual TLS authentication using X.509 certificates
- Publish and subscribe to MQTT topics
- Broadcast incoming messages to multiple receivers
- Configurable MQTT options (keep-alive, packet size, last will, etc.)

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
aws-iot-device-sdk-rust = "0.7"
```

By default the async client is enabled. To use the sync client instead:

```toml
[dependencies]
aws-iot-device-sdk-rust = { version = "0.7", default-features = false, features = ["sync"] }
```

You can also enable both:

```toml
[dependencies]
aws-iot-device-sdk-rust = { version = "0.7", features = ["sync"] }
```

## Prerequisites

To connect to AWS IoT Core you need:

1. An [AWS IoT thing](https://docs.aws.amazon.com/iot/latest/developerguide/iot-moisture-create-thing.html) registered in your account
2. A root CA certificate (e.g. `AmazonRootCA1.pem`)
3. A device certificate (`*.crt`)
4. A private key (`*.pem`)
5. Your AWS IoT endpoint (found in the AWS IoT Core console under **Settings**)

## Usage

### Async client

Create an `AWSIoTAsyncClient`, spawn an event loop listener on a separate task, and use receivers to process incoming messages. Multiple receivers can be created to fan out messages to different parts of your application.

```rust
use aws_iot_device_sdk_rust::{
    async_event_loop_listener, AWSIoTAsyncClient, AWSIoTSettings, Packet, QoS,
};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let aws_settings = AWSIoTSettings::new(
        "my-device-id".to_owned(),
        "AmazonRootCA1.pem".to_owned(),
        "device-cert.crt".to_owned(),
        "private-key.pem".to_owned(),
        "your-endpoint.iot.region.amazonaws.com".to_owned(),
        None,
    );

    let (iot_core_client, eventloop_stuff) = AWSIoTAsyncClient::new(aws_settings).await?;

    iot_core_client.subscribe("my/topic", QoS::AtMostOnce).await?;
    iot_core_client.publish("my/topic", QoS::AtMostOnce, "hello").await?;

    let mut receiver = iot_core_client.get_receiver().await;

    let recv_thread = tokio::spawn(async move {
        loop {
            if let Ok(event) = receiver.recv().await {
                match event {
                    Packet::Publish(p) => {
                        println!("Topic: {}, Payload: {:?}", p.topic, p.payload);
                    }
                    _ => println!("Other event: {:?}", event),
                }
            }
        }
    });

    let listen_thread = tokio::spawn(async move {
        async_event_loop_listener(eventloop_stuff).await.unwrap();
    });

    tokio::join!(recv_thread, listen_thread);
    Ok(())
}
```

### Sync client

The sync client works the same way but uses `std::thread` instead of `tokio`. Enable it with the `sync` feature.

```rust
use aws_iot_device_sdk_rust::{
    sync_client::event_loop_listener, AWSIoTClient, AWSIoTSettings, QoS,
};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let aws_settings = AWSIoTSettings::new(
        "my-device-id".to_owned(),
        "AmazonRootCA1.pem".to_owned(),
        "device-cert.crt".to_owned(),
        "private-key.pem".to_owned(),
        "your-endpoint.iot.region.amazonaws.com".to_owned(),
        None,
    );

    let (mut iot_core_client, event_loop) = AWSIoTClient::new(aws_settings)?;

    std::thread::spawn(move || event_loop_listener(event_loop));

    iot_core_client.subscribe("my/topic/#", QoS::AtMostOnce)?;

    let mut receiver = iot_core_client.get_receiver()?;

    let recv_thread = std::thread::spawn(move || loop {
        if let Ok(event) = receiver.recv() {
            println!("Received: {:?}", event);
        }
    });

    recv_thread.join().unwrap();
    Ok(())
}
```

### MQTT options

You can customize MQTT connection options by passing `MQTTOptionsOverrides` to `AWSIoTSettings`:

```rust
use aws_iot_device_sdk_rust::settings::MQTTOptionsOverrides;
use std::time::Duration;

let mut overrides = MQTTOptionsOverrides::default();
overrides.keep_alive = Some(Duration::from_secs(30));
overrides.clean_session = Some(true);
overrides.conn_timeout = Some(10);

let aws_settings = AWSIoTSettings::new(
    "my-device-id".to_owned(),
    "AmazonRootCA1.pem".to_owned(),
    "device-cert.crt".to_owned(),
    "private-key.pem".to_owned(),
    "your-endpoint.iot.region.amazonaws.com".to_owned(),
    Some(overrides),
);
```

### Using the underlying rumqttc client directly

If you need more control, you can retrieve the underlying rumqttc client and work with it directly:

```rust
let (iot_core_client, (eventloop, _)) = AWSIoTAsyncClient::new(aws_settings).await?;
let client = iot_core_client.get_client().await;
// Use `client` and `eventloop` with the rumqttc API directly
```

## Error handling

All fallible operations return `Result<_, AWSIoTError>`. The error type wraps the underlying rumqttc and I/O errors:

| Variant | Description |
|---|---|
| `AWSConnectionError` | MQTT connection failure (wraps `rumqttc::ConnectionError`) |
| `MQTTClientError` | Client operation failure such as publish/subscribe (wraps `rumqttc::ClientError`) |
| `IoError` | File I/O error when reading certificates or keys |
| `MutexError` | Mutex poisoning (sync client only) |
| `KeyNormalizationError` | Private key normalization failed: non-UTF-8 key data, malformed PEM structure, invalid base64, unrecognized EC curve, or PKCS8 re-encoding failure |


## License

This project is licensed under the [MIT License](LICENSE).
