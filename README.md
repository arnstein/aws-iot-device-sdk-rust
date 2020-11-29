# aws-iot-device-sdk-rust

The AWS IoT Device SDK for Rust allows developers to write Rust to use their devices to access the AWS IoT platform through MQTT.

This repository was forked from [arnstein/aws-iot-device-sdk-rust](https://github.com/arnstein/aws-iot-device-sdk-rust) to add some examples in it.

## Examples

### PubSub

Download your client certificate and private key in `certs` dir as `certificate.pem.crt` and `private.pem.key`.

Edit `src/main_pubsub.rs` and replace `IOT_ENDPOINT` value with yours.

Run the following command to execute the example.

```bash
$ cargo run --bin pubsub
```

## License

MIT
