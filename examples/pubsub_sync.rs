use aws_iot_device_sdk_rust::{sync_client::event_loop_listener, AWSIoTClient, AWSIoTSettings};
use rumqttc::{self, QoS};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let aws_settings = AWSIoTSettings::new(
        "id".to_owned(),
        "/home/myuser/ca".to_owned(),
        "/home/myuser/cert.crt".to_owned(),
        "/home/myuser/key.pem".to_owned(),
        "endpoint.amazonaws.com".to_owned(),
        None,
    );

    let (mut iot_core_client, event_loop) = AWSIoTClient::new(aws_settings)?;

    std::thread::spawn(move || event_loop_listener(event_loop));

    iot_core_client
        .subscribe("/sdk/test/#".to_string(), QoS::AtMostOnce)
        .unwrap();

    let mut receiver1 = iot_core_client.get_receiver();

    let recv1_thread = std::thread::spawn(move || loop {
        if let Ok(event) = receiver1.blocking_recv() {
            println!("Received packet: {event:?}");
        }
    });

    recv1_thread.join().unwrap();

    Ok(())
}
