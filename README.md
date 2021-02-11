[![Documentation](https://docs.rs/aws-iot-device-sdk-rust/badge.svg)](https://docs.rs/aws-iot-device-sdk-rust/)
[![crates.io](https://img.shields.io/crates/v/aws-iot-device-sdk-rust)](https://crates.io/crates/aws-iot-device-sdk-rust)

# aws-iot-device-sdk-rust (unofficial)

The AWS IoT Device SDK for Rust allows developers to write Rust to use their devices to access the AWS IoT platform through MQTT.
This is my first crate, and project, in Rust, and as I am still learning it will hopefully get a lot better with time.
With the client you can publish and subscribe to topics and add callbacks that are associated with topics.
The shadow manager updates, gets, publishes and deletes the device shadow.

It has been through the Works on My Machine Certification Program, and it Works on My Machine™.

This SDK is **unofficial**. Please use it at your own risk.

## Examples

### PubSub

Download [AmazonRootCA1.pem](https://www.amazontrust.com/repository/AmazonRootCA1.pem) into `certs` dir.

Place your client certificate and private key in `certs` dir as `certificate.pem.crt` and `private.pem.key`.

Edit `src/main_pubsub.rs` and replace `IOT_ENDPOINT` value with yours.

Execute following command to run the example.

```bash
$ cargo run --bin pubsub
```

## License

MIT
