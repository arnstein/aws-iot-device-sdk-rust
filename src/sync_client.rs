use crate::error;
use crate::settings::{get_mqtt_options, AWSIoTSettings};
use rumqttc::{
    self, Client, ClientError, Connection, ConnectionError, Event, Packet, QoS, Request,
    Sender as RumqttcSender,
};
use tokio::sync::broadcast;

pub fn event_loop_listener(
    (mut eventloop, incoming_event_sender): (Connection, broadcast::Sender<Packet>),
) -> Result<(), ConnectionError> {
    for notification in eventloop.iter() {
        match notification {
            Ok(event) => {
                if let Event::Incoming(i) = event {
                    if let Err(e) = incoming_event_sender.send(i) {
                        println!("Error sending incoming event: {:?}", e);
                    }
                }
            }
            Err(e) => println!("AWS IoT client error: {:?}", e),
        }
    }
    Ok(())
}

pub struct AWSIoTClient {
    client: Client,
    eventloop_handle: RumqttcSender<Request>,
    event_sender: broadcast::Sender<Packet>,
}

impl AWSIoTClient {
    /// Create new AWSIoTAsyncClient. Input argument should be the AWSIoTSettings. Returns a tuple where the first element is the
    /// AWSIoTAsyncClient, and the second element is a new tuple with the eventloop and incoming
    /// event sender. This tuple should be sent as an argument to the async_event_loop_listener.
    pub fn new(
        settings: AWSIoTSettings,
    ) -> Result<(Self, (Connection, broadcast::Sender<Packet>)), error::AWSIoTError> {
        let mqtt_options = get_mqtt_options(settings)?;

        let (client, connection) = Client::new(mqtt_options, 10);
        let (tx, _) = broadcast::channel(50);
        let eventloop_handle = connection.eventloop.handle();

        let me = Self {
            client,
            eventloop_handle,
            event_sender: tx.clone(),
        };

        Ok((me, (connection, tx)))
    }

    /// Subscribe to a topic.
    pub fn subscribe<S: Into<String>>(&mut self, topic: S, qos: QoS) -> Result<(), ClientError> {
        let res = self.client.subscribe(topic, qos);
        println!("{:?}", res);
        res
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
    pub fn get_receiver(&mut self) -> broadcast::Receiver<Packet> {
        self.event_sender.subscribe()
    }

    /// If you want to use the Rumqttc AsyncClient and EventLoop manually, this method can be used
    /// to get the AsyncClient.
    pub fn get_client(self) -> Client {
        self.client
    }
}
