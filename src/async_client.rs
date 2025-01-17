use crate::error;
use crate::settings::{get_mqtt_options_async, AWSIoTSettings};
use rumqttc::{
    self, AsyncClient, ClientError, ConnectionError, Event, EventLoop, Incoming, NetworkOptions,
    QoS,
};
use tokio::sync::broadcast::{self, Receiver, Sender};

pub async fn async_event_loop_listener(
    (mut eventloop, incoming_event_sender): (EventLoop, Sender<Incoming>),
) -> Result<(), ConnectionError> {
    loop {
        match eventloop.poll().await {
            Ok(event) => {
                if let Event::Incoming(i) = event {
                    if let Err(e) = incoming_event_sender.send(i) {
                        println!("Error sending incoming event: {:?}", e);
                    }
                }
            }
            Err(e) => {
                println!("AWS IoT client error: {:?}", e);
            }
        }
    }
}

pub struct AWSIoTAsyncClient {
    client: AsyncClient,
    incoming_event_sender: Sender<Incoming>,
}

impl AWSIoTAsyncClient {
    /// Create new AWSIoTAsyncClient. Input argument should be the AWSIoTSettings. Returns a tuple where the first element is the
    /// AWSIoTAsyncClient, and the second element is a new tuple with the eventloop and incoming
    /// event sender. This tuple should be sent as an argument to the async_event_loop_listener.
    pub async fn new(
        settings: AWSIoTSettings,
    ) -> Result<(AWSIoTAsyncClient, (EventLoop, Sender<Incoming>)), error::AWSIoTError> {
        let timeout = settings
            .mqtt_options_overrides
            .as_ref()
            .and_then(|opts| opts.conn_timeout);
        let mqtt_options = get_mqtt_options_async(settings).await?;

        let (client, mut eventloop) = AsyncClient::new(mqtt_options, 10);
        if let Some(timeout) = timeout {
            let mut network_options = NetworkOptions::new();
            network_options.set_connection_timeout(timeout);
            eventloop.set_network_options(network_options);
        }

        let (request_tx, _) = broadcast::channel(50);
        Ok((
            AWSIoTAsyncClient {
                client,
                incoming_event_sender: request_tx.clone(),
            },
            (eventloop, request_tx),
        ))
    }

    /// Subscribe to a topic.
    pub async fn subscribe<S: Into<String>>(&self, topic: S, qos: QoS) -> Result<(), ClientError> {
        self.client.subscribe(topic, qos).await
    }

    /// Publish to topic.
    pub async fn publish<S, V>(&self, topic: S, qos: QoS, payload: V) -> Result<(), ClientError>
    where
        S: Into<String>,
        V: Into<Vec<u8>>,
    {
        self.client.publish(topic, qos, false, payload).await
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
