use crate::error;
use rumqttc::{self, LastWill, MqttOptions, TlsConfiguration, Transport};
use std::time::Duration;

const DEFAULT_PORT: u16 = 8883;

#[derive(Clone, Debug)]
pub struct MQTTMaxPacketSize {
    incoming_max_packet_size: usize,
    outgoing_max_packet_size: usize,
}

impl MQTTMaxPacketSize {
    pub fn new(incoming_max_packet_size: usize, outgoing_max_packet_size: usize) -> Self {
        MQTTMaxPacketSize {
            incoming_max_packet_size,
            outgoing_max_packet_size,
        }
    }
}

#[derive(Clone, Default)]
pub struct MQTTOptionsOverrides {
    pub port: Option<u16>,
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
    pub(crate) mqtt_options_overrides: Option<MQTTOptionsOverrides>,
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

fn normalize_key(key_pem: Vec<u8>) -> Result<Vec<u8>, error::AWSIoTError> {
    let key_str = std::str::from_utf8(&key_pem).map_err(|e| {
        error::AWSIoTError::KeyNormalizationError(format!("private key is not valid UTF-8: {e}"))
    })?;

    // Handle SEC1 EC key (BEGIN EC PRIVATE KEY) — extract just the key block,
    // skipping any leading EC PARAMETERS block that would confuse from_sec1_pem.
    let begin_sec1 = "-----BEGIN EC PRIVATE KEY-----";
    let end_sec1 = "-----END EC PRIVATE KEY-----";
    if let Some(start) = key_str.find(begin_sec1) {
        let end_offset = key_str[start..].find(end_sec1).ok_or_else(|| {
            error::AWSIoTError::KeyNormalizationError(
                "SEC1 PEM has BEGIN marker but no END marker".into(),
            )
        })?;
        let ec_block = &key_str[start..start + end_offset + end_sec1.len()];
        if let Ok(key) = p256::SecretKey::from_sec1_pem(ec_block) {
            use p256::pkcs8::EncodePrivateKey;
            return key
                .to_pkcs8_pem(Default::default())
                .map(|doc| doc.as_bytes().to_vec())
                .map_err(|e| {
                    error::AWSIoTError::KeyNormalizationError(format!(
                        "failed to re-encode SEC1 P-256 key as PKCS8: {e}"
                    ))
                });
        }
        if let Ok(key) = p384::SecretKey::from_sec1_pem(ec_block) {
            use p384::pkcs8::EncodePrivateKey;
            return key
                .to_pkcs8_pem(Default::default())
                .map(|doc| doc.as_bytes().to_vec())
                .map_err(|e| {
                    error::AWSIoTError::KeyNormalizationError(format!(
                        "failed to re-encode SEC1 P-384 key as PKCS8: {e}"
                    ))
                });
        }
        return Err(error::AWSIoTError::KeyNormalizationError(
            "SEC1 EC key is not a recognized curve (expected P-256 or P-384)".into(),
        ));
    }

    // Handle PKCS8 EC key (BEGIN PRIVATE KEY) — re-encode through p256/p384 to
    // normalize the structure (e.g. explicit curve params → named curve OID).
    // If it's RSA or Ed25519 PKCS8, parsing will fail and we pass through unchanged.
    let begin_pkcs8 = "-----BEGIN PRIVATE KEY-----";
    let end_pkcs8 = "-----END PRIVATE KEY-----";
    if let Some(start) = key_str.find(begin_pkcs8) {
        let end_offset = key_str[start..].find(end_pkcs8).ok_or_else(|| {
            error::AWSIoTError::KeyNormalizationError(
                "PKCS8 PEM has BEGIN marker but no END marker".into(),
            )
        })?;
        let pkcs8_block = &key_str[start..start + end_offset + end_pkcs8.len()];
        {
            use p256::pkcs8::{DecodePrivateKey, EncodePrivateKey};
            if let Ok(key) = p256::SecretKey::from_pkcs8_pem(pkcs8_block) {
                return key
                    .to_pkcs8_pem(Default::default())
                    .map(|doc| doc.as_bytes().to_vec())
                    .map_err(|e| {
                        error::AWSIoTError::KeyNormalizationError(format!(
                            "failed to re-encode PKCS8 P-256 key: {e}"
                        ))
                    });
            }
        }
        {
            use p384::pkcs8::{DecodePrivateKey, EncodePrivateKey};
            if let Ok(key) = p384::SecretKey::from_pkcs8_pem(pkcs8_block) {
                return key
                    .to_pkcs8_pem(Default::default())
                    .map(|doc| doc.as_bytes().to_vec())
                    .map_err(|e| {
                        error::AWSIoTError::KeyNormalizationError(format!(
                            "failed to re-encode PKCS8 P-384 key: {e}"
                        ))
                    });
            }
        }
        // PKCS8 parsing failed — the PEM header might be lying and the DER
        // content could actually be SEC1 (raw EC key).  Decode the base64
        // payload and try SEC1 DER parsing as a last resort.
        let base64_body: String = pkcs8_block
            .lines()
            .filter(|l| !l.starts_with("-----"))
            .collect();
        use base64::Engine;
        let der = base64::engine::general_purpose::STANDARD
            .decode(base64_body.trim())
            .map_err(|e| {
                error::AWSIoTError::KeyNormalizationError(format!(
                    "PKCS8 PEM contains invalid base64: {e}"
                ))
            })?;
        if let Ok(key) = p256::SecretKey::from_sec1_der(&der) {
            use p256::pkcs8::EncodePrivateKey;
            return key
                .to_pkcs8_pem(Default::default())
                .map(|doc| doc.as_bytes().to_vec())
                .map_err(|e| {
                    error::AWSIoTError::KeyNormalizationError(format!(
                        "failed to re-encode mislabeled SEC1 P-256 key as PKCS8: {e}"
                    ))
                });
        }
        if let Ok(key) = p384::SecretKey::from_sec1_der(&der) {
            use p384::pkcs8::EncodePrivateKey;
            return key
                .to_pkcs8_pem(Default::default())
                .map(|doc| doc.as_bytes().to_vec())
                .map_err(|e| {
                    error::AWSIoTError::KeyNormalizationError(format!(
                        "failed to re-encode mislabeled SEC1 P-384 key as PKCS8: {e}"
                    ))
                });
        }
        // Not an EC key we recognize — pass through unchanged (likely RSA or Ed25519).
        return Ok(key_pem);
    }

    // No recognized PEM header — pass through unchanged (e.g. RSA PRIVATE KEY).
    Ok(key_pem)
}

fn set_overrides(settings: AWSIoTSettings) -> MqttOptions {
    let port = settings
        .mqtt_options_overrides
        .as_ref()
        .map_or(DEFAULT_PORT, |overrides| {
            overrides.port.unwrap_or(DEFAULT_PORT)
        });
    let mut mqtt_options = MqttOptions::new(settings.client_id, settings.aws_iot_endpoint, port);
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
                packet_size.incoming_max_packet_size,
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
        if let Some(last_will) = overrides.last_will {
            mqtt_options.set_last_will(last_will);
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
        let client_key = normalize_key(read(&settings.client_key_path).await?)?;

        Transport::Tls(TlsConfiguration::Simple {
            ca,
            alpn: None,
            client_auth: Some((client_cert, client_key)),
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
        let client_key = normalize_key(read(&settings.client_key_path)?)?;

        Transport::Tls(TlsConfiguration::Simple {
            ca,
            alpn: None,
            client_auth: Some((client_cert, client_key)),
        })
    });

    let mut mqtt_options = set_overrides(settings);
    if let Some(transport) = transport {
        mqtt_options.set_transport(transport);
    }

    Ok(mqtt_options)
}
