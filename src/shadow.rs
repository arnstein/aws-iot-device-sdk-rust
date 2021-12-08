use std::{sync::{Arc, Mutex}, collections::HashMap, str};
use serde_json::{Value, json};
use tokio::sync::{RwLock, broadcast::Receiver};
//use std::sync::RwLock;
use rumqttc::{self, SubscribeFilter, Subscribe, Request, Sender as RumqttcSender, Incoming, Packet, Client, Connection, MqttOptions, Publish, PubAck, QoS, ConnectionError};


pub enum ShadowType {
    Classic,
    Named(String),
}

pub struct AWSShadow {
    device_shadow: RwLock<serde_json::Value>,
    eventloop_handle: RumqttcSender<Request>,
    delete_topic: String,
    delta_topic: String,
    get_topic: String,
}


impl AWSShadow {

    pub async fn new(thing_name: String, shadow_type: ShadowType, eventloop_handle: RumqttcSender<Request>) -> Self {
        let shadow_topic = match shadow_type {
            ShadowType::Classic => format!("$aws/things/{}/shadow", thing_name),
            ShadowType::Named(name) => format!("$aws/things/{}/name/{}", thing_name, name),
        };

        let delete_topic = format!("{}/delete/accepted", shadow_topic);
        let delta_topic = format!("{}/update/delta", shadow_topic);
        let get_topic = format!("{}/get/accepted", shadow_topic);
        let shadow = AWSShadow { delete_topic: delete_topic,
                    delta_topic: delta_topic,
                    get_topic: get_topic.clone(),
                    device_shadow: RwLock::new(serde_json::Value::Null), eventloop_handle: eventloop_handle };

        // subscribe

        shadow.send_packet(get_topic, serde_json::Value::Null).await;
        shadow.subscribe().await;
        shadow
    }

    async fn subscribe(&self) {

        let mut topics = vec![self.delete_topic.clone(), self.delta_topic.clone(), self.get_topic.clone()];
        let mut sub_topic: Vec<SubscribeFilter> = vec![];
        for topic in topics.iter_mut() {
            sub_topic.push(SubscribeFilter::new(topic.clone(), QoS::AtMostOnce));

        }
        let subscribe = Subscribe::new_many(sub_topic);
        let subscribe = Request::Subscribe(subscribe);
        self.eventloop_handle.send(subscribe).await.unwrap();
    }

    async fn send_packet(&self, topic: String, payload: serde_json::Value) {
        let mut publish = Publish::new(topic, QoS::AtMostOnce, payload.to_string());
        publish.retain = false;
        let publish = Request::Publish(publish);
        self.eventloop_handle.send(publish).await.unwrap();
    }

    pub async fn set_shadow(&self, value: serde_json::Value) {
        println!("Setting shadow: {:?}", value.clone());
        let mut shadow = self.device_shadow.write().await;
        *shadow = value.clone();
        self.send_packet(self.delta_topic.clone(), value).await;

    }

    pub async fn get_shadow(&self) -> serde_json::Value {
        let shadow = self.device_shadow.read().await;
        (*shadow).clone()
    }

    pub async fn listen_for_updates(&self, mut receiver: Receiver<Incoming>) {
        let delete_topic = self.delete_topic.clone();
        let get_topic = self.get_topic.clone();
        let delta_topic = self.delta_topic.clone();
        loop {
            match receiver.recv().await {
                Ok(event) => {
                    match event {
                        Packet::Publish(p) => {
                            println!("Event: {:?}", p);
                            let topic = p.topic.as_str();
                            let payload: Value = serde_json::from_slice(&(p.payload.clone())).unwrap();
                            if topic.contains(&get_topic) || topic.contains(&delta_topic) {
                                self.set_shadow(payload).await;
                            }
                            if topic.contains(&delete_topic) {
                                self.set_shadow(serde_json::Value::Null).await;
                            }
                        }
                    _ => (),
                    }
                },
                Err(_) => (),
            }
        }
    }
}
