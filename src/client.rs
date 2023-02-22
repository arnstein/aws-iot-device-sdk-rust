use std::fs::read;
use tokio::{sync::broadcast::{self, Receiver, Sender}, time::Duration};
use rumqttc::{self, Event, Key, Transport, TlsConfiguration, Incoming, LastWill, MqttOptions, QoS, ConnectionError};
use rumqttc::{Sender as RumqttcSender, Request, ClientError};
use crate::error;

#[cfg(feature= "async")]
use rumqttc::{EventLoop, AsyncClient};

pub struct MQTTMaxPacketSize {
    icoming_max_packet_size: usize,
    outgoing_max_packet_size: usize,
}

pub struct MQTTOptionsOverrides {
    pub clean_session: Option<bool>,
    pub keep_alive: Option<Duration>,
    pub max_packet_size: Option<MQTTMaxPacketSize>,
    pub request_channel_capacity: Option<usize>,
    pub pending_throttle: Option<Duration>,
    pub inflight: Option<u16>,
    pub last_will: Option<LastWill>,
    pub conn_timeout: Option<u64>,
    pub transport: Option<Transport>
}

pub struct AWSIoTSettings {
        client_id: String,
        ca_path: String,
        client_cert_path: String,
        client_key_path: String,
        aws_iot_endpoint: String,
        mqtt_options_overrides: Option<MQTTOptionsOverrides>
}

impl AWSIoTSettings {
    pub fn new(
        client_id: String,
        ca_path: String,
        client_cert_path: String,
        client_key_path: String,
        aws_iot_endpoint: String,
        mqtt_options_overrides: Option<MQTTOptionsOverrides>
    ) -> AWSIoTSettings {
        AWSIoTSettings {
            client_id,
            ca_path,
            client_cert_path,
            client_key_path,
            aws_iot_endpoint,
            mqtt_options_overrides
        }
    }
}

fn get_mqtt_options(settings: AWSIoTSettings) -> Result<MqttOptions, error::AWSIoTError> {
    let mut mqtt_options = MqttOptions::new(settings.client_id, settings.aws_iot_endpoint, 8883);
    let mut transport_overrided = false;

    if let Some(overrides) = settings.mqtt_options_overrides {
        if let Some(clean_session) = overrides.clean_session {
            mqtt_options.set_clean_session(clean_session);
        }
        if let Some(transport) = overrides.transport {
            transport_overrided = true;
            mqtt_options.set_transport(transport);
        }
        if let Some(keep_alive) = overrides.keep_alive {
            mqtt_options.set_keep_alive(keep_alive);
        }
        if let Some(packet_size) = overrides.max_packet_size {
            mqtt_options.set_max_packet_size(packet_size.icoming_max_packet_size, packet_size.outgoing_max_packet_size);
        }
        if let Some(request_channel_capacity) = overrides.request_channel_capacity {
            mqtt_options.set_request_channel_capacity(request_channel_capacity);
        }
        if let Some(pending_throttle) = overrides.pending_throttle {
            mqtt_options.set_pending_throttle(pending_throttle);
        }
        if let Some(inflight) = overrides.inflight {
            mqtt_options.set_inflight(inflight);
        }
        if let Some(clean_session) = overrides.clean_session {
            mqtt_options.set_clean_session(clean_session);
        }
        if let Some(last_will) = overrides.last_will {
            mqtt_options.set_last_will(last_will);
        }
        if let Some(conn_timeout) = overrides.conn_timeout {
            mqtt_options.set_connection_timeout(conn_timeout);
        }
    }

    if !transport_overrided {
        let ca = read(settings.ca_path)?;
        let client_cert = read(settings.client_cert_path)?;
        let client_key = read(settings.client_key_path)?;

        let transport = Transport::Tls(TlsConfiguration::Simple {
            ca: ca.to_vec(),
            alpn: None,
            client_auth: Some((client_cert.to_vec(), Key::RSA(client_key.to_vec()))),
        });
        mqtt_options.set_transport(transport);
    }

    mqtt_options.set_keep_alive(Duration::from_secs(10));

    Ok(mqtt_options)

}

pub async fn async_event_loop_listener((mut eventloop, incoming_event_sender): (EventLoop, Sender<Incoming>)) -> Result<(), ConnectionError>{
    loop {
        match eventloop.poll().await {
            Ok(event) => {
                match event {
                    Event::Incoming(i) => {
                        match incoming_event_sender.send(i) {
                            Ok(_) => (),
                            Err(e) => println!("Error sending incoming event: {:?}", e),
                        }
                    },
                    _ => (),
                }
            },
            Err(e) => {
                println!("AWS IoT client error: {:?}", e);
            }
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
