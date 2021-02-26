use std::{collections::HashMap, str};
use serde_json::{Value, json};
use rumqttc::{self, Incoming, Client, Connection, MqttOptions, Publish, PubAck, QoS, ConnectionError};


pub trait AWSShadow {
    item: Value,
}
