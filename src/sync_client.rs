use crate::error;
use crate::settings::{get_mqtt_options, AWSIoTSettings};
use bus::{Bus, BusReader};
use rumqttc::{
    self, Client, ClientError, Connection, ConnectionError, Event, EventLoop, Incoming, QoS,
    Request, Sender as RumqttcSender,
};

pub fn event_loop_listener(
    (mut eventloop, mut incoming_event_sender): (Connection, std::sync::mpsc::Sender<Incoming>),
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
    incoming_event_sender: std::sync::mpsc::Sender<Incoming>,
    event_receiver: BusReader<Incoming>,
}

impl AWSIoTClient {
    /// Create new AWSIoTAsyncClient. Input argument should be the AWSIoTSettings. Returns a tuple where the first element is the
    /// AWSIoTAsyncClient, and the second element is a new tuple with the eventloop and incoming
    /// event sender. This tuple should be sent as an argument to the async_event_loop_listener.
    pub fn new(
        settings: AWSIoTSettings,
    ) -> Result<(Self, (Connection, std::sync::mpsc::Sender<Incoming>)), error::AWSIoTError> {
        let mqtt_options = get_mqtt_options(settings)?;

        let (client, connection) = Client::new(mqtt_options, 10);
        let (multi_sender, single_consumer) = std::sync::mpsc::channel();
        let mut sender = bus::Bus::new(50);
        let event_receiver = sender.add_rx();
        std::thread::spawn(move || {
            while let Ok(msg) = single_consumer.recv() {
                sender.broadcast(msg);
            }
        });
        let eventloop_handle = connection.eventloop.handle();
        Ok((
            Self {
                client,
                eventloop_handle,
                incoming_event_sender: multi_sender.clone(),
                event_receiver,
            },
            (connection, multi_sender),
        ))
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
    pub fn get_receiver(&mut self) -> BusReader<Incoming> {
        self.incoming_event_sender.add_rx()
    }

    /// If you want to use the Rumqttc AsyncClient and EventLoop manually, this method can be used
    /// to get the AsyncClient.
    pub fn get_client(self) -> Client {
        self.client
    }
}
