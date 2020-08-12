use std::{fs::read, thread, collections::HashMap, sync::{Arc, Mutex}};
use std::time::{Duration};
use async_channel::Sender;
use rumqttc::{self, Incoming, Client, Connection, EventLoop, MqttOptions, Publish, QoS, Subscribe, Request, ConnectionError};
use async_trait::async_trait;

#[async_trait]
pub trait AWSEventHandler {

    fn on_connect() {}
    fn on_publish(message: Publish) {}

    async fn start_async_event_listener(&mut self, mut eventloop: EventLoop) {
        //loop {
        let (incoming, outgoing) = eventloop.poll().await.unwrap();
        match incoming {
            Some(Incoming::Publish(message)) => {
                Self::on_publish(message);
            },
            Some(Incoming::Connected) => {
                Self::on_connect();
            },
            _ => (),
            None => (),
        }
        //}
    }

    fn start_event_listener(&self, mut connection: Connection) {
        for notification in connection.iter() {
            match notification {
                Ok(notification_type) => match notification_type.0 {
                    Some(Incoming::Publish(message)) => {
                        Self::on_publish(message);
                    },
                    Some(Incoming::Connected) => {
                        Self::on_connect();
                    },
                    _ => (),
                    None => (),
                },
                Err(_) => (),
            }
        }
    }
}

pub struct AWSIoTSyncClient {
    pub client: Client,
}

impl AWSIoTSyncClient {
    pub fn new(
        client_id: &str,
        ca_path: &str,
        client_cert_path: &str,
        client_key_path: &str,
        aws_iot_endpoint: &str) -> Result<(AWSIoTSyncClient, Connection), ConnectionError> {

        let mut mqtt_options = MqttOptions::new(client_id, aws_iot_endpoint, 8883);
        mqtt_options.set_ca(read(ca_path)?)
            .set_client_auth(read(client_cert_path)?, read(client_key_path)?)
            .set_keep_alive(10);

            let (client, connection) = Client::new(mqtt_options, 10);
            Ok((AWSIoTSyncClient { client: client }, connection))
    }

    /// Subscribe to any topic.
    pub fn subscribe (&mut self, topic_name: String, qos: QoS) {
        self.client.subscribe(topic_name, qos).unwrap();
    }

    /// Publish to any topic.
    pub fn publish (&mut self, topic_name: String, qos: QoS, payload: &str) {
        self.client.publish(topic_name, qos, false, payload).unwrap();
    }
}

pub struct AWSIoTAsyncClient {
    pub sender: Sender<Request>,
}

impl AWSIoTAsyncClient {

    pub async fn new(
        client_id: &str,
        ca_path: &str,
        client_cert_path: &str,
        client_key_path: &str,
        aws_iot_endpoint: &str) -> Result<(AWSIoTAsyncClient, EventLoop), ConnectionError> {

        let mut mqtt_options = MqttOptions::new(client_id, aws_iot_endpoint, 8883);
        mqtt_options.set_ca(read(ca_path)?)
            .set_client_auth(read(client_cert_path)?, read(client_key_path)?)
            .set_keep_alive(10);
        let eventloop = EventLoop::new(mqtt_options, 10).await;
        let requests_tx = eventloop.handle();
        Ok((AWSIoTAsyncClient { sender: requests_tx }, eventloop))
    }

    /// Subscribe to any topic.
    pub async fn subscribe (&mut self, topic_name: String, qos: QoS) {
        let subscribe = Subscribe::new(topic_name, qos);
        self.sender.send(Request::Subscribe(subscribe)).await.unwrap();
    }

    /// Publish to any topic.
    pub async fn publish (&mut self, topic_name: String, qos: QoS, payload: &str) {
        let publish = Publish::new(topic_name, qos, payload);
        self.sender.send(Request::Publish(publish)).await.unwrap();
    }
}
