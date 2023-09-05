use crate::error;
use rumqttc::{self, Key, LastWill, MqttOptions, TlsConfiguration, Transport};
use std::time::Duration;

pub struct MQTTMaxPacketSize {
    icoming_max_packet_size: usize,
    outgoing_max_packet_size: usize,
}

pub struct MQTTOptionsOverrides {
    pub clean_session: Option<bool>,
    pub keep_alive: Option<Duration>,
    pub max_packet_size: Option<MQTTMaxPacketSize>,
    pub request_channel_capacity: Option<usize>,
    pub pending_throttle: Option<Duration>,
    pub inflight: Option<u16>,
    pub last_will: Option<LastWill>,
    pub conn_timeout: Option<u64>,
    pub transport: Option<Transport>,
}

pub struct AWSIoTSettings {
    client_id: String,
    ca_path: String,
    client_cert_path: String,
    client_key_path: String,
    aws_iot_endpoint: String,
    mqtt_options_overrides: Option<MQTTOptionsOverrides>,
}

impl AWSIoTSettings {
    pub fn new(
        client_id: String,
        ca_path: String,
        client_cert_path: String,
        client_key_path: String,
        aws_iot_endpoint: String,
        mqtt_options_overrides: Option<MQTTOptionsOverrides>,
    ) -> AWSIoTSettings {
        AWSIoTSettings {
            client_id,
            ca_path,
            client_cert_path,
            client_key_path,
            aws_iot_endpoint,
            mqtt_options_overrides,
        }
    }
}

fn set_overrides(settings: AWSIoTSettings) -> MqttOptions {
    let mut mqtt_options = MqttOptions::new(settings.client_id, settings.aws_iot_endpoint, 8883);
    mqtt_options.set_keep_alive(Duration::from_secs(10));
    if let Some(overrides) = settings.mqtt_options_overrides {
        if let Some(clean_session) = overrides.clean_session {
            mqtt_options.set_clean_session(clean_session);
        }
        if let Some(transport) = overrides.transport {
            mqtt_options.set_transport(transport);
        }
        if let Some(keep_alive) = overrides.keep_alive {
            mqtt_options.set_keep_alive(keep_alive);
        }
        if let Some(packet_size) = overrides.max_packet_size {
            mqtt_options.set_max_packet_size(
                packet_size.icoming_max_packet_size,
                packet_size.outgoing_max_packet_size,
            );
        }
        if let Some(request_channel_capacity) = overrides.request_channel_capacity {
            mqtt_options.set_request_channel_capacity(request_channel_capacity);
        }
        if let Some(pending_throttle) = overrides.pending_throttle {
            mqtt_options.set_pending_throttle(pending_throttle);
        }
        if let Some(inflight) = overrides.inflight {
            mqtt_options.set_inflight(inflight);
        }
        if let Some(clean_session) = overrides.clean_session {
            mqtt_options.set_clean_session(clean_session);
        }
        if let Some(last_will) = overrides.last_will {
            mqtt_options.set_last_will(last_will);
        }
        if let Some(conn_timeout) = overrides.conn_timeout {
            mqtt_options.set_connection_timeout(conn_timeout);
        }
    }


    mqtt_options
}

#[cfg(feature = "async")]
pub(crate) async fn get_mqtt_options_async(
    settings: AWSIoTSettings,
) -> Result<MqttOptions, error::AWSIoTError> {
    use tokio::fs::read;

    let transport_overrided = settings
        .mqtt_options_overrides
        .as_ref()
        .is_some_and(|over| over.transport.is_some());

    let transport = (!transport_overrided).then_some({
        let ca = read(&settings.ca_path).await?;
        let client_cert = read(&settings.client_cert_path).await?;
        let client_key = read(&settings.client_key_path).await?;

        Transport::Tls(TlsConfiguration::Simple {
            ca,
            alpn: None,
            client_auth: Some((client_cert, Key::RSA(client_key))),
        })
    });

    let mut mqtt_options = set_overrides(settings);
    if let Some(transport) = transport {
        mqtt_options.set_transport(transport);
    }

    Ok(mqtt_options)
}

#[cfg(feature = "sync")]
pub(crate) fn get_mqtt_options(
    settings: AWSIoTSettings,
) -> Result<MqttOptions, error::AWSIoTError> {
    use std::fs::read;

    let transport_overrided = settings
        .mqtt_options_overrides
        .as_ref()
        .is_some_and(|over| over.transport.is_some());

    let transport = (!transport_overrided).then_some({
        let ca = read(&settings.ca_path)?;
        let client_cert = read(&settings.client_cert_path)?;
        let client_key = read(&settings.client_key_path)?;

        Transport::Tls(TlsConfiguration::Simple {
            ca,
            alpn: None,
            client_auth: Some((client_cert, Key::RSA(client_key))),
        })
    });

    let mut mqtt_options = set_overrides(settings);
    if let Some(transport) = transport {
        mqtt_options.set_transport(transport);
    }

    Ok(mqtt_options)
}
