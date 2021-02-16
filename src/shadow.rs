use crate::client::AWSIoTClient;
use rumqttc::QoS;
use std::sync::{Arc, Mutex};

pub struct AWSShadowManager<'a> {
    pub aws_iot_client: &'a mut AWSIoTClient,
    pub thing_name: String,
    device_shadow: Arc<Mutex<serde_json::Value>>,
}

impl<'a> AWSShadowManager<'a> {
    /// Returns an instantiated shadow manager
    pub fn new(aws_iot_client: &'a mut AWSIoTClient, thing_name: String) -> Self {
        AWSShadowManager {
            aws_iot_client: aws_iot_client,
            thing_name: thing_name,
            device_shadow: Arc::new(Mutex::new(serde_json::Value::Null)),
        }
    }

    /// Subscribe to get/accepted topic
    pub fn subscribe_to_get_shadow_accepted(&mut self, callback: fn(String)) {
        let topic = format!("$aws/things/{}/shadow/get/accepted", self.thing_name);
        self.aws_iot_client
            .subscribe(topic, QoS::AtMostOnce, callback);
    }

    /// Subscribe to get/rejected topic
    pub fn subscribe_to_get_shadow_rejected(&mut self, callback: fn(String)) {
        let topic = format!("$aws/things/{}/shadow/get/rejected", self.thing_name);
        self.aws_iot_client
            .subscribe(topic, QoS::AtMostOnce, callback);
    }

    /// Subscribe to update/accepted topic
    pub fn subscribe_to_update_shadow_accepted(&mut self, callback: fn(String)) {
        let topic = format!("$aws/things/{}/shadow/update/accepted", self.thing_name);
        self.aws_iot_client
            .subscribe(topic, QoS::AtMostOnce, callback);
    }

    /// Subscribe to update/rejected topic
    pub fn subscribe_to_update_shadow_rejected(&mut self, callback: fn(String)) {
        let topic = format!("$aws/things/{}/shadow/update/rejected", self.thing_name);
        self.aws_iot_client
            .subscribe(topic, QoS::AtMostOnce, callback);
    }

    /// Subscribe to update/delta topic
    pub fn subscribe_to_update_shadow_delta(&mut self, callback: fn(String)) {
        let topic = format!("$aws/things/{}/shadow/update/delta", self.thing_name);
        self.aws_iot_client
            .subscribe(topic, QoS::AtMostOnce, callback);
    }

    /// Subscribe to update/documents topic
    pub fn subscribe_to_update_shadow_documents(&mut self, callback: fn(String)) {
        let topic = format!("$aws/things/{}/shadow/update/documents", self.thing_name);
        self.aws_iot_client
            .subscribe(topic, QoS::AtMostOnce, callback);
    }

    /// Subscribe to delete/accepted topic
    pub fn subscribe_to_delete_shadow_accepted(&mut self, callback: fn(String)) {
        let topic = format!("$aws/things/{}/shadow/delete/accepted", self.thing_name);
        self.aws_iot_client
            .subscribe(topic, QoS::AtMostOnce, callback);
    }

    /// Subscribe to delete/rejected topic
    pub fn subscribe_to_delete_shadow_rejected(&mut self, callback: fn(String)) {
        let topic = format!("$aws/things/{}/shadow/delete/rejected", self.thing_name);
        self.aws_iot_client
            .subscribe(topic, QoS::AtMostOnce, callback);
    }

    /// Posts an empty message to get the device shadow from the AWS IoT Core broker.
    pub fn get_shadow(&mut self) {
        let topic = format!("$aws/things/{}/shadow/get", self.thing_name);
        self.aws_iot_client.publish(topic, QoS::AtMostOnce, "");
    }

    /// Update a value in the local shadow, then publish it.
    pub fn update_shadow(&mut self, key: &str, value: serde_json::Value) {
        let shadow = Arc::clone(&self.device_shadow);
        shadow.lock().unwrap()["state"]["reported"][key] = value;
        let topic = String::from(format!("$aws/things/{}/shadow/update", self.thing_name));
        self.aws_iot_client
            .publish(topic, QoS::AtMostOnce, &shadow.lock().unwrap().to_string());
    }

    /// Deletes shadow.
    pub fn delete_shadow(&mut self) {
        let topic = format!("$aws/things/{}/shadow/delete", self.thing_name);
        self.aws_iot_client.publish(topic, QoS::AtMostOnce, "");
    }
}
