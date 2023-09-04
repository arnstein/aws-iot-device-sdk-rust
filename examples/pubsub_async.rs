use aws_iot_device_sdk_rust::{async_event_loop_listener, AWSIoTAsyncClient, AWSIoTSettings};
use rumqttc::{self, Packet, QoS};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let aws_settings = AWSIoTSettings::new(
        "id".to_owned(),
        "/home/myuser/ca".to_owned(),
        "/home/myuser/cert.crt".to_owned(),
        "/home/myuser/key.pem".to_owned(),
        "endpoint.amazonaws.com".to_owned(),
        None,
    );

    let (iot_core_client, eventloop_stuff) = AWSIoTAsyncClient::new(aws_settings).await?;

    iot_core_client.subscribe("test".to_string(), QoS::AtMostOnce).await.unwrap();
    iot_core_client.publish("topic".to_string(), QoS::AtMostOnce, "hey").await.unwrap();

    let mut receiver1 = iot_core_client.get_receiver().await;
    let mut receiver2 = iot_core_client.get_receiver().await;

    let recv1_thread = tokio::spawn(async move {
        loop {
            if let Ok(event) = receiver1.recv().await {
                match event {
                    Packet::Publish(p) => {
                        println!("Received message {:?} on topic: {}", p.payload, p.topic)
                    }
                    _ => println!("Got event on receiver1: {:?}", event),
                }
            }
        }
    });

    let recv2_thread = tokio::spawn(async move {
        loop {
            if let Ok(event) = receiver2.recv().await {
                println!("Got event on receiver2: {:?}", event);
            }
        }
    });
    let listen_thread = tokio::spawn(async move {
        async_event_loop_listener(eventloop_stuff).await.unwrap();
        //iot_core_client.listen().await.unwrap();
    });

    //iot_core_client.publish("topic".to_string(), QoS::AtMostOnce, "hey").await.unwrap();
    tokio::join!(recv1_thread, recv2_thread, listen_thread);

    Ok(())
}
