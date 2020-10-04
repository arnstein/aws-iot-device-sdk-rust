use std::fs::read;
use rumqttc::{self, Event, Packet, Incoming, Client, LastWill, Connection, MqttOptions, Publish, PubAck, SubAck, UnsubAck, QoS, ConnectionError};

#[cfg(feature= "async")]
use rumqttc::{EventLoop, AsyncClient, Subscribe, Request};
use async_channel::Sender;

pub trait AWSEventHandler {

    fn on_connect(&self) {}
    fn on_publish(&self, message: Publish) {}
    fn on_puback(&self, puback: PubAck) {}
    fn on_suback(&self, suback: SubAck) {}
    fn on_unsuback(&self, unsuback: UnsubAck) {}
    fn on_pingreq(&self) {}
    fn on_pingresp(&self) {}

}

//fn start_event_listener(mut connection: Connection, aws_struct: Box<dyn AWSEventHandler>) {
//fn start_event_listener<T: AWSEventHandler + ?Sized> (mut connection: Connection, aws_struct: &T) {
//    for notification in connection.iter() {
//        match notification {
//            Ok(notification_type) => match notification_type.0 {
//                Some(Incoming::Connect(c)) => {
//                    aws_struct.on_connect();
//                },
//                Some(Incoming::Publish(message)) => {
//                    aws_struct.on_publish(message);
//                },
//                Some(Incoming::PubAck(puback)) => {
//                    aws_struct.on_puback(puback);
//                },
//                Some(Incoming::SubAck(suback)) => {
//                    aws_struct.on_suback(suback);
//                },
//                Some(Incoming::UnsubAck(unsuback)) => {
//                    aws_struct.on_unsuback(unsuback);
//                },
//                Some(Incoming::PingReq) => {
//                    aws_struct.on_pingreq();
//                },
//                Some(Incoming::PingResp) => {
//                    aws_struct.on_pingresp();
//                },
//                _ => (),
//            },
//            Err(_) => (),
//        }
//    }
//}
#[cfg(feature= "async")]
async fn start_async_event_listener<T: AWSEventHandler + ?Sized> (mut eventloop: EventLoop, aws_struct: &T) -> Result<(), ConnectionError>{

    loop {
        match eventloop.poll().await? {
            Event::Incoming(i) => {
                match i {
                    Incoming::Connect(c) => aws_struct.on_connect(),
                    Incoming::Publish(message) => {
                        aws_struct.on_publish(message);
                    },
                    Incoming::PubAck(puback) => {
                        aws_struct.on_puback(puback);
                    },
                    Incoming::SubAck(suback) => {
                        aws_struct.on_suback(suback);
                    },
                    Incoming::UnsubAck(unsuback) => {
                        aws_struct.on_unsuback(unsuback);
                    },
                    Incoming::PingReq => {
                        aws_struct.on_pingreq();
                    },
                    Incoming::PingResp => {
                        aws_struct.on_pingresp();
                    },
            }
        }
    }
}

pub struct AWSIoTSettings {
        client_id: String,
        ca_path: String,
        client_cert_path: String,
        client_key_path: String,
        aws_iot_endpoint: String,
}

impl AWSIoTSettings {
    pub fn new(
        client_id: String,
        ca_path: String,
        client_cert_path: String,
        client_key_path: String,
        aws_iot_endpoint: String) -> AWSIoTSettings {

        AWSIoTSettings { client_id, ca_path, client_cert_path, client_key_path, aws_iot_endpoint }
    }
}

pub struct AWSIoTClient {
    pub client: Client,
}

impl AWSIoTClient {
    pub fn new(
        settings: AWSIoTSettings
        ) -> Result<(AWSIoTClient, Connection), ConnectionError> {

        let mut mqtt_options = MqttOptions::new(settings.client_id, settings.aws_iot_endpoint, 8883);
        mqtt_options.set_ca(read(settings.ca_path)?)
            .set_client_auth(read(settings.client_cert_path)?, read(settings.client_key_path)?)
            .set_keep_alive(10);

            let (client, connection) = Client::new(mqtt_options, 10);
            Ok((AWSIoTClient { client: client }, connection))
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

#[cfg(feature= "async")]
pub struct AWSIoTAsyncClient {
    pub client: AsyncClient,
    pub eventloop: EventLoop,
}

#[cfg(feature= "async")]
impl AWSIoTAsyncClient {

    pub async fn new(
        client_id: &str,
        ca_path: &str,
        client_cert_path: &str,
        client_key_path: &str,
        aws_iot_endpoint: &str,
        last_will: Option<LastWill>) -> Result<(AWSIoTAsyncClient, EventLoop), ConnectionError> {

        let mut mqtt_options = MqttOptions::new(client_id, aws_iot_endpoint, 8883);
        mqtt_options.set_ca(read(ca_path)?)
            .set_client_auth(read(client_cert_path)?, read(client_key_path)?)
            .set_keep_alive(10);

        match last_will {
            Some(last_will) => {
                mqtt_options.set_last_will(last_will);
            },
            None => (),
        }

        let (client, eventloop) = AsyncClient::new(mqtt_options, 10);
        Ok((AWSIoTAsyncClient { client, eventloop }))
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
