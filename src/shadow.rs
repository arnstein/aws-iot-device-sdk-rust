use std::{sync::{Arc, Mutex}, collections::HashMap, str};
use serde_json::{Value, json};
use tokio::sync::broadcast::Receiver;
//use std::sync::RwLock;
use std::sync::RwLock;
use rumqttc::{self, Request, Sender as RumqttcSender, Incoming, Packet, Client, Connection, MqttOptions, Publish, PubAck, QoS, ConnectionError};


enum ShadowType {
    Classic,
    Named(String),
}

struct AWSShadow {
    shadow_topic: String,
    device_shadow: RwLock<serde_json::Value>,
    eventloop_handle: RumqttcSender<Request>,
}

impl AWSShadow {

    pub fn new(thing_name: String, shadow_type: ShadowType, eventloop_handle: RumqttcSender<Request>) -> Self {
        let shadow_topic = match shadow_type {
            ShadowType::Classic => format!("$aws/things/{}/shadow", thing_name),
            ShadowType::Named(name) => format!("$aws/things/{}/name/{}", thing_name, name),
        };
        // send empty message to /get to get shadow

        AWSShadow { shadow_topic: shadow_topic,
                    device_shadow: RwLock::new(serde_json::Value::Null), eventloop_handle: eventloop_handle }

    }

    pub fn set_shadow(&self, value: serde_json::Value) {
        let mut shadow = self.device_shadow.write().unwrap();
        // send shadow to /update
        *shadow = value;
    }

    pub fn get_shadow(&self) -> serde_json::Value {
        let shadow = self.device_shadow.read().unwrap();
        (*shadow).clone()
    }

    pub async fn listen_for_updates(&self, mut receiver: Receiver<Incoming>) {
        loop {
            match receiver.recv().await {
                Ok(event) => {
                    match event {
                        Packet::Publish(p) => {
                            match p.topic {
                                format!("{}/delete/accepted", self.shadow_topic) => self.set_shadow(serde_json::Value::Null),
                                format!("{}/get/accepted", self.shadow_topic)  => self.set_shadow(p.payload),
                                format!("{}/update/delta", self.shadow_topic) => self.set_shadow(p.payload),
                                _ => (),
                            }
                        }
                    _ => (),
                    },
                },
                Err(_) => (),
            }
        }
    }

}
