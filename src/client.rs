use rumqtt::{
    ConnectError, MqttClient, MqttOptions, Notification, QoS, Receiver, ReconnectOptions,
};
use std::{
    collections::HashMap,
    fs::read,
    sync::{Arc, Mutex},
    thread,
};

pub struct AWSIoTClient {
    pub aws_iot_client: MqttClient,
    receiver: Receiver<Notification>,
    callback_map: Arc<Mutex<HashMap<String, fn(String)>>>,
    is_listening: bool,
}

impl AWSIoTClient {
    /// Returns an AWSIoTClient struct with the instantiated MQTT client ready to be used.
    pub fn new(
        client_id: &str,
        ca_path: &str,
        client_cert_path: &str,
        client_key_path: &str,
        aws_iot_endpoint: &str,
    ) -> Result<Self, ConnectError> {
        let mqtt_options = MqttOptions::new(client_id, aws_iot_endpoint, 8883)
            .set_ca(read(ca_path)?)
            .set_client_auth(read(client_cert_path)?, read(client_key_path)?)
            .set_keep_alive(10)
            .set_reconnect_opts(ReconnectOptions::Always(5));
        let mqtt_client = MqttClient::start(mqtt_options)?;
        Ok(AWSIoTClient {
            aws_iot_client: mqtt_client.0,
            receiver: mqtt_client.1,
            callback_map: Arc::new(Mutex::new(HashMap::new())),
            is_listening: false,
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

    /// When called it will spawn a thread that checks if the HashMap in the AWSIoTClient contain
    /// a callback function associated with the incoming topics.
    pub fn start_listening(&mut self) {
        let callback_map = Arc::clone(&self.callback_map);
        let receiver = self.receiver.clone();
        thread::spawn(move || loop {
            for notification in &receiver {
                match notification {
                    rumqtt::client::Notification::Publish(packet) => {
                        match callback_map.lock().unwrap().get(&packet.topic_name) {
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

    /// Subscribe to any topic.
    pub fn subscribe(&mut self, topic_name: String, qos: QoS, callback: fn(String)) {
        self.aws_iot_client.subscribe(&topic_name, qos).unwrap();
        if !self.is_listening {
            self.add_callback(topic_name, callback);
            self.start_listening();
            self.is_listening = true;
        }
    }

    // Unsubscribe from a topic.
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
