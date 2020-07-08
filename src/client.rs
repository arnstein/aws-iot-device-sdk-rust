use std::{fs::read, thread, collections::HashMap, sync::{Arc, Mutex}};
use crossbeam_channel::select;
use tokio::sync::mpsc::{channel, Sender, Receiver};
//use rumqtt::{QoS, ConnectError, ReconnectOptions, Receiver, Notification, MqttClient, MqttOptions};
use rumqttc::{self, EventLoop, MqttOptions, Publish, QoS, Subscribe, Request, ConnectionError};

pub trait AWSEventHandler {

    fn on_disconnect(payload: Vec<u8>) {}
    fn on_reconnect(payload: Vec<u8>) {}
    fn on_publish(payload: Vec<u8>) {}

    fn start_event_listener(&self, eventloop: &EventLoop<Receiver<Request>>) {
    }
}
pub struct AWSIoTClient {
    pub eventloop: EventLoop<Receiver<Request>>,
    pub sender: Sender<Request>,
}

impl AWSIoTClient {

    pub async fn new(
        client_id: &str,
        ca_path: &str,
        client_cert_path: &str,
        client_key_path: &str,
        aws_iot_endpoint: &str) -> Result<AWSIoTClient, ConnectionError> {

        let mut mqtt_options = MqttOptions::new(client_id, aws_iot_endpoint, 8883);
        mqtt_options.set_ca(read(ca_path)?)
            .set_client_auth(read(client_cert_path)?, read(client_key_path)?)
            .set_keep_alive(10);
        let (requests_tx, requests_rx) = channel(10);
        let mut eventloop = EventLoop::new(mqtt_options, requests_rx).await;
        Ok(AWSIoTClient { eventloop: eventloop, sender: requests_tx })
    }

    /// Subscribe to any topic.
    pub async fn subscribe (&mut self, topic_name: String, qos: QoS) {
        let subscribe = Subscribe::new("test", QoS::AtMostOnce);
        self.sender.send(Request::Subscribe(subscribe)).await.unwrap();
    }

    /// Publish to any topic.
    pub async fn publish (&mut self, topic_name: String, qos: QoS, payload: &str) {
        let publish = Publish::new("test", QoS::AtLeastOnce, vec![]);
        self.sender.send(Request::Publish(publish)).await.unwrap();
    }
}
