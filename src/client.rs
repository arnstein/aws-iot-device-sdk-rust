use std::fs::read;
use tokio::{sync::broadcast::{self, Receiver, Sender}, time::Duration};
use rumqttc::{self, Event, Key, Transport, TlsConfiguration, Incoming, Client, LastWill, Connection, MqttOptions, Publish, QoS, ConnectionError};
use rumqttc::{Sender as RumqttcSender, Request, ClientError};
use crate::error;

#[cfg(feature= "async")]
use rumqttc::{EventLoop, AsyncClient};

pub struct AWSIoTSettings {
        client_id: String,
        ca_path: String,
        client_cert_path: String,
        client_key_path: String,
        aws_iot_endpoint: String,
        last_will: Option<LastWill>,
}

impl AWSIoTSettings {
    pub fn new(
        client_id: String,
        ca_path: String,
        client_cert_path: String,
        client_key_path: String,
        aws_iot_endpoint: String,
        last_will: Option<LastWill>) -> AWSIoTSettings {

        AWSIoTSettings {
            client_id,
            ca_path,
            client_cert_path,
            client_key_path,
            aws_iot_endpoint,
            last_will }
    }
}

fn get_mqtt_options(settings: AWSIoTSettings) -> Result<MqttOptions, error::AWSIoTError> {
    let mut mqtt_options = MqttOptions::new(settings.client_id, settings.aws_iot_endpoint, 8883);
    let ca = read(settings.ca_path)?;
    let client_cert = read(settings.client_cert_path)?;
    let client_key = read(settings.client_key_path)?;

    let transport = Transport::Tls(TlsConfiguration::Simple {
        ca: ca.to_vec(),
        alpn: None,
        client_auth: Some((client_cert.to_vec(), Key::RSA(client_key.to_vec()))),
    });
    mqtt_options.set_transport(transport)
        .set_keep_alive(Duration::from_secs(10));

    match settings.last_will {
        Some(last_will) => {
            mqtt_options.set_last_will(last_will);
        },
        None => (),
    }

    Ok(mqtt_options)

}

pub struct AWSIoTClient {
    client: Client,
    incoming_event_sender: Sender<Incoming>,
}


impl AWSIoTClient {

    pub fn new(
        settings: AWSIoTSettings
        ) -> Result<(AWSIoTClient, (Connection, Sender<Incoming>)), ConnectionError> {

        let mqtt_options = get_mqtt_options(settings).unwrap();

        let (client, connection) = Client::new(mqtt_options, 10);
        let (request_tx, _) = broadcast::channel(16);
        Ok((AWSIoTClient { client: client,
                                incoming_event_sender: request_tx.clone() },
                                (connection, request_tx)))
    }

    pub fn subscribe<S: Into<String>>(&mut self, topic: S, qos: QoS) -> Result<(), ClientError> {
        self.client.subscribe(topic, qos).unwrap();
        Ok(())
    }

    pub fn publish<S, V>(&mut self, topic: S, qos: QoS, payload: V) -> Result<(), ClientError>
    where
        S: Into<String>,
        V: Into<Vec<u8>>,
    {
        self.client.publish(topic, qos, false, payload).unwrap();
        Ok(())
    }

    pub fn get_receiver(&self) -> Receiver<Incoming> {
        self.incoming_event_sender.subscribe()
    }

    pub fn get_client(self) -> Client {
        self.client
    }
}

pub struct AWSIoTAsyncClient {
    client: AsyncClient,
    eventloop_handle: RumqttcSender<Request>,
    incoming_event_sender: Sender<Incoming>,
}


impl AWSIoTAsyncClient {

    pub async fn new(
        settings: AWSIoTSettings
        ) -> Result<(AWSIoTAsyncClient, (EventLoop, Sender<Incoming>)), ConnectionError> {

        let mqtt_options = get_mqtt_options(settings).unwrap();

        let (client, eventloop) = AsyncClient::new(mqtt_options, 10);
        let (request_tx, _) = broadcast::channel(16);
        let eventloop_handle = eventloop.handle();
        Ok((AWSIoTAsyncClient { client: client,
                                eventloop_handle: eventloop_handle,
                                incoming_event_sender: request_tx.clone() },
                                (eventloop, request_tx)))
    }

    pub async fn subscribe<S: Into<String>>(&self, topic: S, qos: QoS) -> Result<(), ClientError> {
        self.client.subscribe(topic, qos).await.unwrap();
        Ok(())
    }

    pub async fn publish<S, V>(&self, topic: S, qos: QoS, payload: V) -> Result<(), ClientError> 
    where
        S: Into<String>,
        V: Into<Vec<u8>>,
    {
        self.client.publish(topic, qos, false, payload).await.unwrap();
        Ok(())
    }

    pub async fn get_eventloop_handle(&self) -> RumqttcSender<Request> {
        self.eventloop_handle.clone()
    }

    pub async fn get_receiver(&self) -> Receiver<Incoming> {
        self.incoming_event_sender.subscribe()
    }

    pub async fn get_client(self) -> AsyncClient {
        self.client
    }

}

pub async fn async_event_loop_listener((mut eventloop, incoming_event_sender): (EventLoop, Sender<Incoming>)) -> Result<(), ConnectionError>{
    loop {
        match eventloop.poll().await? {
            Event::Incoming(i) => {
                incoming_event_sender.send(i).unwrap();
            },
            _ => (),
        }
    }
}

pub fn event_loop_listener((mut connection, incoming_event_sender): (Connection, Sender<Incoming>)) -> Result<(), ConnectionError>{
    for (i, notification) in connection.iter().enumerate() {
        match notification.unwrap() {
            Event::Incoming(i) => {
                incoming_event_sender.send(i).unwrap();
            },
            _ => (),
        }
    }
    Ok(())
}
