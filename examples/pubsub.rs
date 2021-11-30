use serde_json::json;
use rumqttc::{self, Incoming, Client, Connection, EventLoop, MqttOptions, Publish, QoS, Subscribe, Request, ConnectionError};
use async_channel::{Sender as AsyncSender, Receiver as AsyncReceiver, unbounded as asyncunbounded};
use async_trait::async_trait;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
//fn main() {

    let aws_settings = client::AWSIoTSettings::new(
        "id".to_owned(),
        "/home/myuser/ca".to_owned(),
        "/home/myuser/cert.crt".to_owned(),
        "/home/myuser/key.pem".to_owned(),
        "endpoint.amazonaws.com".to_owned(),
        None
        );

    let mut iot_core_client = client::AWSIoTAsyncClient::new(aws_settings).await?;
    loop {
        iot_core_client.listen().await;
    }
    Ok(())
}
