use crate::error;
use crate::settings::{get_mqtt_options, AWSIoTSettings};
use bus::{Bus, BusReader};
use log::error;
use rumqttc::{self, Client, ClientError, Connection, ConnectionError, Event, Packet, QoS};
use std::sync::{Arc, Mutex};

pub type EventBus = Arc<Mutex<Bus<Packet>>>;

pub fn event_loop_listener(
    (mut connection, event_bus): (Connection, EventBus),
) -> Result<(), Box<ConnectionError>> {
    for notification in connection.iter() {
        match notification {
            Ok(event) => {
                if let Event::Incoming(i) = event {
                        let mut bus = event_bus.lock().expect("event bus mutex poisoned");
                    bus.broadcast(i);
                }
            }
            Err(e) => error!("AWS IoT client error: {:?}", e),
        }
    }
    Ok(())
}

pub struct AWSIoTClient {
    client: Client,
    event_bus: Arc<Mutex<Bus<Packet>>>,
}

impl AWSIoTClient {
    /// Create new AWSIoTClient. Input argument should be the AWSIoTSettings. Returns a tuple where the first element is the
    /// AWSIoTClient, and the second element is a new tuple with the connection and event bus.
    /// This tuple should be sent as an argument to the event_loop_listener.
    pub fn new(
        settings: AWSIoTSettings,
    ) -> Result<(Self, (Connection, EventBus)), error::AWSIoTError> {
        let mqtt_options = get_mqtt_options(settings)?;

        let (client, connection) = Client::new(mqtt_options, 10);
        let event_bus = Arc::new(Mutex::new(Bus::new(50)));

        let me = Self {
            client,
            event_bus: Arc::clone(&event_bus),
        };

        Ok((me, (connection, event_bus)))
    }

    /// Subscribe to a topic.
    pub fn subscribe<S: Into<String>>(&mut self, topic: S, qos: QoS) -> Result<(), ClientError> {
        self.client.subscribe(topic, qos)
    }

    /// Publish to topic.
    pub fn publish<S, V>(&mut self, topic: S, qos: QoS, payload: V) -> Result<(), ClientError>
    where
        S: Into<String>,
        V: Into<Vec<u8>>,
    {
        self.client.publish(topic, qos, false, payload)
    }

    /// Get a receiver of the incoming messages. Send this to any function that wants to read the
    /// incoming messages from IoT Core.
    pub fn get_receiver(&mut self) -> BusReader<Packet> {
        self.event_bus.lock().expect("Failed to lock event bus").add_rx()
    }

    /// If you want to use the Rumqttc Client and Connection manually, this method can be used
    /// to get the Client.
    pub fn get_client(self) -> Client {
        self.client
    }
}
