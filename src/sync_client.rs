use crate::error::AWSIoTError;
use crate::settings::{get_mqtt_options, AWSIoTSettings};
use bus::{Bus, BusReader};
use rumqttc::{self, Client, Connection, Event, Packet, QoS};
use std::sync::{Arc, Mutex};

pub type EventBus = Arc<Mutex<Bus<Packet>>>;

pub fn event_loop_listener(
    (mut connection, event_bus): (Connection, EventBus),
) -> Result<(), AWSIoTError> {
    for notification in connection.iter() {
        match notification {
            Ok(event) => {
                if let Event::Incoming(i) = event {
                    let mut bus = event_bus.lock()?;
                    bus.broadcast(i);
                }
            }
            Err(e) => return Err(e.into()),
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
    ) -> Result<(Self, (Connection, EventBus)), AWSIoTError> {
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
    pub fn subscribe<S: Into<String>>(
        &mut self,
        topic: S,
        qos: QoS,
    ) -> Result<(), AWSIoTError> {
        self.client.subscribe(topic, qos).map_err(Into::into)
    }

    /// Publish to topic.
    pub fn publish<S, V>(
        &mut self,
        topic: S,
        qos: QoS,
        payload: V,
    ) -> Result<(), AWSIoTError>
    where
        S: Into<String>,
        V: Into<Vec<u8>>,
    {
        self.client
            .publish(topic, qos, false, payload)
            .map_err(Into::into)
    }

    /// Get a receiver of the incoming messages. Send this to any function that wants to read the
    /// incoming messages from IoT Core.
    pub fn get_receiver(&mut self) -> Result<BusReader<Packet>, AWSIoTError> {
        Ok(self.event_bus.lock()?.add_rx())
    }

    /// If you want to use the Rumqttc Client and Connection manually, this method can be used
    /// to get the Client.
    pub fn get_client(self) -> Client {
        self.client
    }
}
