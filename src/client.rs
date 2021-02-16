use rumqttc::{
    Client, Connection, ConnectionError, Event, Incoming, Key, MqttOptions, QoS, Transport,
};
use std::{
    collections::HashMap,
    fs::read,
    sync::{Arc, Mutex},
    thread,
};

pub struct AWSIoTClient {
    pub aws_iot_client: Client,
    callback_map: Arc<Mutex<HashMap<String, fn(String)>>>,
}

impl AWSIoTClient {
    /// Returns an AWSIoTClient struct with the instantiated MQTT client ready to be used.
    pub fn new(
        client_id: &str,
        ca_path: &str,
        client_cert_path: &str,
        client_key_path: &str,
        aws_iot_endpoint: &str,
    ) -> Result<Self, ConnectionError> {
        let mut mqtt_options = MqttOptions::new(client_id, aws_iot_endpoint, 8883);
        mqtt_options
            .set_keep_alive(10)
            .set_transport(Transport::tls(
                read(ca_path)?,
                Some((read(client_cert_path)?, Key::RSA(read(client_key_path)?))),
                None,
            ));
        let (client, connection) = Client::new(mqtt_options, 10);
        let callback_map = Arc::new(Mutex::new(HashMap::new()));
        start_listening(connection, Arc::clone(&callback_map));
        Ok(AWSIoTClient {
            aws_iot_client: client,
            callback_map: callback_map,
        })
    }

    /// Associates a callback function with a topic name.
    pub fn add_callback(&self, topic_name: String, callback: fn(String)) -> Option<fn(String)> {
        self.callback_map
            .lock()
            .unwrap()
            .insert(topic_name, callback)
    }

    /// Remove the callback function associated with a topic name.
    pub fn remove_callback(&self, topic_name: String) -> Option<fn(String)> {
        self.callback_map.lock().unwrap().remove(&topic_name)
    }

    /// Subscribe to any topic.
    pub fn subscribe(&mut self, topic_name: String, qos: QoS, callback: fn(String)) {
        self.aws_iot_client.subscribe(&topic_name, qos).unwrap();
        self.add_callback(topic_name, callback);
    }

    /// UnSubscribe from a topic.
    pub fn unsubscribe(&mut self, topic_name: String) {
        self.aws_iot_client.unsubscribe(&topic_name).unwrap();
        self.remove_callback(topic_name);
    }

    /// Publish to any topic.
    pub fn publish(&mut self, topic_name: String, qos: QoS, payload: &str) {
        self.aws_iot_client
            .publish(topic_name, qos, false, payload)
            .unwrap();
    }
}

/// When called it will spawn a thread that checks if the HashMap in the AWSIoTClient contain
/// a callback function associated with the incoming topics.
fn start_listening(
    mut connection: Connection,
    callback_map: Arc<Mutex<HashMap<String, fn(String)>>>,
) {
    thread::spawn(move || loop {
        for notification in connection.iter() {
            match notification {
                Ok(Event::Incoming(Incoming::Publish(packet))) => {
                    match callback_map.lock().unwrap().get(&packet.topic) {
                        Some(&func) => {
                            func(String::from_utf8(packet.payload.to_vec()).unwrap());
                        }
                        _ => (),
                    }
                }
                _ => (),
            }
        }
    });
}
