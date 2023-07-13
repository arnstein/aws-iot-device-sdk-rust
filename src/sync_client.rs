use crate::error;
use crate::settings::{get_mqtt_options, AWSIoTSettings};
use bus::BusReader;
use rumqttc::{self, Client, ClientError, Incoming, QoS, Request, Sender as RumqttcSender};

pub struct AWSIoTClient {
    client: Client,
    eventloop_handle: RumqttcSender<Request>,
    incoming_event_sender: bus::Bus<Incoming>,
}

impl AWSIoTClient {
    /// Create new AWSIoTAsyncClient. Input argument should be the AWSIoTSettings. Returns a tuple where the first element is the
    /// AWSIoTAsyncClient, and the second element is a new tuple with the eventloop and incoming
    /// event sender. This tuple should be sent as an argument to the async_event_loop_listener.
    pub fn new(settings: AWSIoTSettings) -> Result<Self, error::AWSIoTError> {
        let mqtt_options = get_mqtt_options(settings)?;

        let (client, connection) = Client::new(mqtt_options, 10);
        let sender = bus::Bus::new(50);
        let eventloop_handle = connection.eventloop.handle();
        Ok(Self {
            client,
            eventloop_handle,
            incoming_event_sender: sender,
        })
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

    /// Get an eventloop handle that can be used to interract with the eventloop. Not needed if you
    /// are only using client.publish and client.subscribe.
    pub fn get_eventloop_handle(&self) -> RumqttcSender<Request> {
        self.eventloop_handle.clone()
    }

    /// Get a receiver of the incoming messages. Send this to any function that wants to read the
    /// incoming messages from IoT Core.
    pub fn get_receiver(&mut self) -> BusReader<Incoming> {
        self.incoming_event_sender.add_rx()
    }

    /// If you want to use the Rumqttc AsyncClient and EventLoop manually, this method can be used
    /// to get the AsyncClient.
    pub fn get_client(self) -> Client {
        self.client
    }
}
