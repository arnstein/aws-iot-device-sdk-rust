use std::fs::read;
use tokio::{sync::broadcast::{self, Receiver, Sender}, time::Duration};
use rumqttc::{self, Event, Key, Transport, TlsConfiguration, Incoming, LastWill, MqttOptions, QoS, ConnectionError};
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

pub async fn async_event_loop_listener((mut eventloop, incoming_event_sender): (EventLoop, Sender<Incoming>)) -> Result<(), ConnectionError>{
    loop {
        match eventloop.poll().await? {
            Event::Incoming(i) => {
                match incoming_event_sender.send(i) {
                    Ok(_) => (),
                    Err(e) => println!("Error sending incoming event: {:?}", e),
                }
            },
            _ => (),
        }
    }
}

pub struct AWSIoTAsyncClient {
    client: AsyncClient,
    eventloop_handle: RumqttcSender<Request>,
    incoming_event_sender: Sender<Incoming>,
}


impl AWSIoTAsyncClient {

    /// Create new AWSIoTAsyncClient. Input argument should be the AWSIoTSettings. Returns a tuple where the first element is the
    /// AWSIoTAsyncClient, and the second element is a new tuple with the eventloop and incoming
    /// event sender. This tuple should be sent as an argument to the async_event_loop_listener.
    pub async fn new(
        settings: AWSIoTSettings
        ) -> Result<(AWSIoTAsyncClient, (EventLoop, Sender<Incoming>)), ConnectionError> {

        let mqtt_options = get_mqtt_options(settings).unwrap();

        let (client, eventloop) = AsyncClient::new(mqtt_options, 10);
        let (request_tx, _) = broadcast::channel(50);
        let eventloop_handle = eventloop.handle();
        Ok((AWSIoTAsyncClient { client: client,
                                eventloop_handle: eventloop_handle,
                                incoming_event_sender: request_tx.clone() },
                                (eventloop, request_tx)))
    }

    /// Subscribe to a topic.
    pub async fn subscribe<S: Into<String>>(&self, topic: S, qos: QoS) -> Result<(), ClientError> {
        self.client.subscribe(topic, qos).await.unwrap();
        Ok(())
    }

    /// Publish to topic.
    pub async fn publish<S, V>(&self, topic: S, qos: QoS, payload: V) -> Result<(), ClientError> 
    where
        S: Into<String>,
        V: Into<Vec<u8>>,
    {
        self.client.publish(topic, qos, false, payload).await.unwrap();
        Ok(())
    }

    /// Get an eventloop handle that can be used to interract with the eventloop. Not needed if you
    /// are only using client.publish and client.subscribe.
    pub async fn get_eventloop_handle(&self) -> RumqttcSender<Request> {
        self.eventloop_handle.clone()
    }

    /// Get a receiver of the incoming messages. Send this to any function that wants to read the
    /// incoming messages from IoT Core.
    pub async fn get_receiver(&self) -> Receiver<Incoming> {
        self.incoming_event_sender.subscribe()
    }

    /// If you want to use the Rumqttc AsyncClient and EventLoop manually, this method can be used
    /// to get the AsyncClient.
    pub async fn get_client(self) -> AsyncClient {
        self.client
    }

}
