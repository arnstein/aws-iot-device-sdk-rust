mod client;

use rumqttc::QoS;
use std::{thread, time};

fn callback(payload: String) {
    println!("Receive: {}", payload);
}

fn main() {
    const CLIENT_ID: &str = "test-device";
    const CA_CERT: &str = "certs/AmazonRootCA1.pem";
    const CLIENT_CERT: &str = "certs/certificate.pem.crt";
    const PRIVATE_KEY: &str = "certs/private.pem.key";
    const IOT_ENDPOINT: &str = "REPLACE-HERE.iot.REGION.amazonaws.com";

    let iot_client =
        &mut client::AWSIoTClient::new(CLIENT_ID, CA_CERT, CLIENT_CERT, PRIVATE_KEY, IOT_ENDPOINT)
            .unwrap();
    println!("Connect: {}", IOT_ENDPOINT);

    const TOPIC: &str = "data/test";

    iot_client.subscribe(TOPIC.to_string(), QoS::AtMostOnce, callback);
    thread::sleep(time::Duration::from_secs(1));

    for i in 0..10 {
        let payload = format!("{{\"test\": \"Hello world {}.\"}}", i);
        println!("Publish: {}", payload);
        iot_client.publish(TOPIC.to_string(), QoS::AtMostOnce, &payload);
        thread::sleep(time::Duration::from_secs(1));
    }

    iot_client.unsubscribe(TOPIC.to_string());
}
