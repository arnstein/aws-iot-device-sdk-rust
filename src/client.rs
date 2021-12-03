use std::fs::read;
use tokio::{sync::broadcast::{self, Receiver, Sender}, time::Duration};
use rumqttc::{self, Event, Key, Transport, TlsConfiguration, Incoming, Client, LastWill, Connection, MqttOptions, Publish, QoS, ConnectionError};
use rumqttc::{Sender as RumqttcSender, Connect, PubAck, SubAck, UnsubAck, PubRec, PubRel, PubComp, Subscribe, Unsubscribe, Disconnect, ConnAck, Request, ClientError};
use crate::error;
//use async_channel::{unbounded, Sender, Receiver};
//use crossbeam_channel::{unbounded, Sender, Receiver};

#[cfg(feature= "async")]
use rumqttc::{EventLoop, AsyncClient};

//pub trait AWSEventHandler {
//
//    fn on_connect(&mut self, connect: Connect) {}
//    fn on_connack(&mut self, connect_info: ConnAck) {}
//    fn on_publish(&mut self, message: Publish) {}
//    fn on_puback(&self, puback: PubAck) {}
//    fn on_pubrec(&self, pubrec: PubRec) {}
//    fn on_pubrel(&self, pubrel: PubRel) {}
//    fn on_pubcomp(&self, pubcomp: PubComp) {}
//    fn on_subscribe(&self, subscribe: Subscribe) {}
//    fn on_unsubscribe(&self, unsubscribe: Unsubscribe) {}
//    fn on_disconnect(&self) {}
//    fn on_suback(&self, suback: SubAck) {}
//    fn on_unsuback(&self, unsuback: UnsubAck) {}
//    fn on_pingreq(&self) {}
//    fn on_pingresp(&self) {}
//}
//
//async fn incoming_event_handler<T: AWSEventHandler + ?Sized> (receiver:  &mut Receiver<Event>, aws_struct: &mut T) {
//    let message = receiver.recv().unwrap();
//        match message {
//            Event::Incoming(i) => {
//                match i {
//                    Incoming::Connect(connect) => aws_struct.on_connect(connect),
//                    Incoming::Publish(message) => {
//                        aws_struct.on_publish(message);
//                    },
//                    Incoming::PubAck(puback) => {
//                        aws_struct.on_puback(puback);
//                    },
//                    Incoming::SubAck(suback) => {
//                        aws_struct.on_suback(suback);
//                    },
//                    Incoming::UnsubAck(unsuback) => {
//                        aws_struct.on_unsuback(unsuback);
//                    },
//                    Incoming::PingReq => {
//                        aws_struct.on_pingreq();
//                    },
//                    Incoming::PingResp => {
//                        aws_struct.on_pingresp();
//                    },
//                    Incoming::ConnAck(c) => {
//                        aws_struct.on_connack(c);
//                    },
//                    Incoming::PubRec(pubrec) => {
//                        aws_struct.on_pubrec(pubrec);
//                    },
//                    Incoming::PubRel(pubrel) => {
//                        aws_struct.on_pubrel(pubrel);
//                    },
//                    Incoming::PubComp(pubcomp) => {
//                        aws_struct.on_pubcomp(pubcomp);
//                    },
//                    Incoming::Subscribe(subscribe) => {
//                        aws_struct.on_subscribe(subscribe);
//                    },
//                    Incoming::Unsubscribe(unsubscribe) => {
//                        aws_struct.on_unsubscribe(unsubscribe);
//                    },
//                    Incoming::Disconnect => {
//                        aws_struct.on_disconnect();
//                    },
//                }
//            },
//            _ => (),
//        }
//    }

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

#[cfg(feature= "async")]
pub struct AWSIoTAsyncClient {
    client: AsyncClient,
    eventloop_handle: RumqttcSender<Request>,
    incoming_event_sender: Sender<Incoming>,
}


#[cfg(feature= "async")]
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

    //pub async fn get_client_and_eventloop(self) -> (AsyncClient, EventLoop) {
    //    (self.client, self.eventloop)
    //}

    //pub async fn listen(&mut self) -> Result<(), ConnectionError>{
    //    loop {
    //        match self.eventloop.poll().await? {
    //            Event::Incoming(i) => {
    //                self.incoming_event_sender.send(i).unwrap();
    //            },
    //            _ => (),
    //            // => println!("Got: {:?}"),
    //        }
    //    }
    //}
}

pub async fn listen((mut eventloop, incoming_event_sender): (EventLoop, Sender<Incoming>)) -> Result<(), ConnectionError>{
    loop {
        match eventloop.poll().await? {
            Event::Incoming(i) => {
                incoming_event_sender.send(i).unwrap();
            },
            _ => (),
            // => println!("Got: {:?}"),
        }
    }
}
