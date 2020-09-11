use std::{collections::HashMap, str};
use serde_json::{Value, json};
use rumqttc::{self, Incoming, Client, Connection, MqttOptions, Publish, PubAck, QoS, ConnectionError};


pub trait AWSShadow {
    const ID: String;

    fn on_shadow_update(shadow: Value) {}
    fn on_shadow_get(shadow: Value) {}
    fn get_shadow() {}

    fn start_shadow_listener(&self, mut connection: Connection) {
        let shadow_get_topic = format!("$aws/things/{}/shadow/get/accepted", Self::ID).to_string();
        let shadow_update_topic = format!("$aws/things/{}/shadow/update/accepted", Self::ID).to_string();
        for notification in connection.iter() {
            match notification {
                Ok(notification_type) => match notification_type.0 {
                    Some(Incoming::Publish(message)) => {
                        match message.topic {
                            shadow_update_topic => {
                                let shadow: Value = serde_json::from_str(str::from_utf8(&message.payload).unwrap()).unwrap();
                                Self::on_shadow_update(shadow)
                            },
                            shadow_get_topic => {
                                let shadow: Value = serde_json::from_str(str::from_utf8(&message.payload).unwrap()).unwrap();
                                Self::on_shadow_get(shadow)
                            },
                            _ => (),
                        }
                    },
                    _ => (),
                },
                Err(_) => (),
            }
        }
    }
}
