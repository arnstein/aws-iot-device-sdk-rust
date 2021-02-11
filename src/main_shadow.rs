mod client;
mod shadow;

use serde_json::json;
use serde_json::Value;
use std::{thread, time};

fn callback_get_accepted(payload: String) {
    println!("Get Shadow Accepted: {}", payload);
}

fn callback_get_rejected(payload: String) {
    println!("Get Shadow Rejected: {}", payload);
}

fn callback_update_accepted(payload: String) {
    println!("Update Shadow Accepted: {}", payload);
}

fn callback_update_rejected(payload: String) {
    println!("Update Shadow Rejected: {}", payload);
}

fn callback_update_delta(payload: String) {
    println!("Update Shadow Delta: {}", payload);
    let delta: Value = serde_json::from_str(&payload).unwrap();
    for (key, value) in delta["state"].as_object().unwrap() {
        println!("Delta: {}: {}", key, value);
    }
}

fn callback_delete_accepted(payload: String) {
    println!("Delete Shadow Accepted: {}", payload);
}

fn callback_delete_rejected(payload: String) {
    println!("Delete Shadow Rejected: {}", payload);
}

fn main() {
    const THING_NAME: &str = "test-device";
    const CA_CERT: &str = "certs/AmazonRootCA1.pem";
    const CLIENT_CERT: &str = "certs/certificate.pem.crt";
    const PRIVATE_KEY: &str = "certs/private.pem.key";
    const IOT_ENDPOINT: &str = "REPLACE-HERE.iot.REGION.amazonaws.com";

    let iot_client =
        &mut client::AWSIoTClient::new(THING_NAME, CA_CERT, CLIENT_CERT, PRIVATE_KEY, IOT_ENDPOINT)
            .unwrap();
    println!("Connect: {}", IOT_ENDPOINT);

    let shadow_manager = &mut shadow::AWSShadowManager::new(iot_client, THING_NAME.to_string());

    println!("Subscribe to Shadow related topics");
    shadow_manager.subscribe_to_get_shadow_accepted(callback_get_accepted);
    shadow_manager.subscribe_to_get_shadow_rejected(callback_get_rejected);
    shadow_manager.subscribe_to_update_shadow_accepted(callback_update_accepted);
    shadow_manager.subscribe_to_update_shadow_rejected(callback_update_rejected);
    shadow_manager.subscribe_to_update_shadow_delta(callback_update_delta);
    shadow_manager.subscribe_to_delete_shadow_accepted(callback_delete_accepted);
    shadow_manager.subscribe_to_delete_shadow_rejected(callback_delete_rejected);
    thread::sleep(time::Duration::from_secs(1));

    println!("Get Shadow");
    shadow_manager.get_shadow();
    thread::sleep(time::Duration::from_secs(1));

    println!("Update Shadow");
    let value: Value = json!("value");
    shadow_manager.update_shadow("test", value);
    thread::sleep(time::Duration::from_secs(1));

    loop {
        thread::sleep(time::Duration::from_secs(1));
    }
}
