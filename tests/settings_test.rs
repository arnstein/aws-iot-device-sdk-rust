use aws_iot_device_sdk_rust::settings::{AWSIoTSettings, MQTTMaxPacketSize, MQTTOptionsOverrides};
use rumqttc::LastWill;
use std::time::Duration;

#[test]
fn new_settings_stores_fields() {
    let settings = AWSIoTSettings::new(
        "client1".to_owned(),
        "ca.pem".to_owned(),
        "cert.crt".to_owned(),
        "key.pem".to_owned(),
        "endpoint.amazonaws.com".to_owned(),
        None,
    );
    // Settings created without panic — fields are private, so we verify via construction.
    drop(settings);
}

#[test]
fn new_settings_with_overrides() {
    let overrides = MQTTOptionsOverrides {
        port: Some(443),
        clean_session: Some(false),
        keep_alive: Some(Duration::from_secs(30)),
        max_packet_size: Some(MQTTMaxPacketSize::new(1024, 2048)),
        request_channel_capacity: Some(100),
        pending_throttle: Some(Duration::from_millis(500)),
        inflight: Some(5),
        last_will: Some(LastWill::new("lwt", "offline", rumqttc::QoS::AtLeastOnce, false)),
        conn_timeout: Some(10),
        transport: None,
    };

    let settings = AWSIoTSettings::new(
        "client2".to_owned(),
        "ca.pem".to_owned(),
        "cert.crt".to_owned(),
        "key.pem".to_owned(),
        "endpoint.amazonaws.com".to_owned(),
        Some(overrides),
    );
    drop(settings);
}

#[test]
fn default_overrides_are_all_none() {
    let overrides = MQTTOptionsOverrides::default();
    assert!(overrides.port.is_none());
    assert!(overrides.clean_session.is_none());
    assert!(overrides.keep_alive.is_none());
    assert!(overrides.max_packet_size.is_none());
    assert!(overrides.request_channel_capacity.is_none());
    assert!(overrides.pending_throttle.is_none());
    assert!(overrides.inflight.is_none());
    assert!(overrides.last_will.is_none());
    assert!(overrides.conn_timeout.is_none());
    assert!(overrides.transport.is_none());
}

#[test]
fn max_packet_size_stores_values() {
    let pkt = MQTTMaxPacketSize::new(4096, 8192);
    // Clone works
    let _cloned = pkt.clone();
    // Debug works
    let debug = format!("{:?}", pkt);
    assert!(debug.contains("4096"));
    assert!(debug.contains("8192"));
}
