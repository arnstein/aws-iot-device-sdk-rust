use rumqtt::QoS;
use crate::client::AWSIoTClient;
use std::{sync::{Arc, Mutex}};

pub struct AWSShadowManager<'a> {
    pub aws_iot_client: &'a mut AWSIoTClient,
    pub thing_name: String,
    device_shadow: Arc<Mutex<serde_json::Value>>,
}

impl<'a> AWSShadowManager<'a> {

    /// Returns an instantiated shadow manager
    pub fn new(aws_iot_client: &'a mut AWSIoTClient, thing_name: String) -> Self {
        AWSShadowManager { aws_iot_client: aws_iot_client, thing_name: thing_name, device_shadow: Arc::new(Mutex::new(serde_json::Value::Null)) }
    }

    /// Posts an empty message to get the device shadow from the AWS IoT Core broker.
    pub fn get_shadow (&mut self, callback: fn(String)){
        let shadow_get = format!("$aws/things/{}/shadow/get", self.thing_name);
        let shadow_get_cb = format!("$aws/things/{}/shadow/get/accepted", self.thing_name);
        self.aws_iot_client.add_callback(shadow_get_cb, callback);
        self.aws_iot_client.publish(shadow_get, QoS::AtMostOnce, "");
    }

    /// Deletes shadow.
    pub fn delete_shadow (&mut self, callback: fn(String)) {
        let shadow_topic = format!("$aws/things/{}/shadow/delete/accepted", self.thing_name);
        self.aws_iot_client.add_callback(shadow_topic.clone(), callback);
        self.aws_iot_client.publish(shadow_topic, QoS::AtMostOnce, "{}");
    }

    /// Add a callback for when a change to the shadow is accepted.
    pub fn add_listen_on_delta_callback(&mut self, callback: fn(String)) {
        let shadow_topic = String::from(format!("$aws/things/{}/shadow/update/accepted", self.thing_name));
        self.aws_iot_client.subscribe(shadow_topic.clone(), QoS::AtMostOnce);
        self.aws_iot_client.add_callback(shadow_topic, callback);
    }

    /// Call this if you do not longer want to have a callback associated with the shadow update
    /// topic.
    pub fn remove_listen_on_delta_callback(&mut self) {
        let shadow_topic = String::from(format!("$aws/things/{}/shadow/update/accepted", self.thing_name));
        self.aws_iot_client.remove_callback(shadow_topic);
    }

    /// Subscribes to all shadow related topics.
    pub fn subscribe_all (&mut self) {
        let shadow_base_url = format!("$aws/things/{}/shadow", self.thing_name);
        let shadow_topics = vec!["update/accepted", "update/documents", "update/rejected", "update/delta", "get/accepted", "get/rejected", "delete/accepted", "delete/rejected"];
        for topic in &shadow_topics {
            self.aws_iot_client.subscribe(format!("{}/{}", shadow_base_url, topic), QoS::AtLeastOnce);
        }
    }

    /// Update a value in the local shadow, then publish it.
    pub fn update_shadow(&mut self, key: &str, value: serde_json::Value) {
        let shadow = Arc::clone(&self.device_shadow);
        shadow.lock().unwrap()["state"]["reported"][key] = value;
        let shadow_topic = String::from(format!("$aws/things/{}/shadow/update", self.thing_name));
        self.aws_iot_client.publish(shadow_topic, QoS::AtMostOnce, &shadow.lock().unwrap().to_string());
    }

}
