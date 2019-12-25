use std::{fs::read, thread, collections::HashMap, sync::{Arc, Mutex}};
use rumqtt::{QoS, ReconnectOptions, Receiver, Notification, MqttClient, MqttOptions};

pub struct AWSIoTClient {
    pub aws_iot_client: MqttClient,
    receiver: Receiver<Notification>,
    callback_map: Arc<Mutex<HashMap<String, fn(String)>>>,
}

impl AWSIoTClient {

    /// Returns an AWSIoTClient struct with the instantiated MQTT client ready to be used.
    pub fn new(
        client_id: &str,
        ca_path: &str,
        client_cert_path: &str,
        client_key_path: &str,
        aws_iot_endpoint: &str) -> AWSIoTClient {

        let mqtt_options = MqttOptions::new(client_id, aws_iot_endpoint, 8883)
            .set_ca(read(ca_path).unwrap())
            .set_client_auth(read(client_cert_path).unwrap(), read(client_key_path).unwrap())
            .set_keep_alive(10)
            .set_reconnect_opts(ReconnectOptions::Always(5));
        let mqtt_client = MqttClient::start(mqtt_options).unwrap();

        AWSIoTClient { aws_iot_client: mqtt_client.0, receiver: mqtt_client.1, callback_map: Arc::new(Mutex::new(HashMap::new())) }
    }

    /// Associates a callback function with a topic name.
    pub fn add_callback(&self, topic_name: String, callback: fn(String)) {
        self.callback_map
            .lock()
            .unwrap()
            .insert(topic_name, callback);
    }

    /// Remove the callback function associated with a topic name.
    pub fn remove_callback(&self, topic_name: String) -> Option<fn(String)> {
        self.callback_map
            .lock()
            .unwrap()
            .remove(&topic_name)
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
                            match callback_map
                                .lock()
                                .unwrap()
                                .get(&packet.topic_name) {
                                    Some(&func) => {
                                    func(String::from_utf8(packet.payload.to_vec()).unwrap());
                                }
                                _ => (),
                            }
                        },
                        _ => (),
                    }
                }
        });
    }

    /// Subscribe to any topic.
    pub fn subscribe (&mut self, topic_name: String, qos: QoS) {
            self.aws_iot_client.subscribe(topic_name, qos).unwrap();
    }

    /// Publish to any topic.
    pub fn publish (&mut self, topic_name: String, qos: QoS, payload: &str) {
            self.aws_iot_client.publish(topic_name, qos, false, payload).unwrap();
    }
}
