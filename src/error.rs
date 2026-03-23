use rumqttc::{ClientError, ConnectionError};
use std::fmt;
use std::fmt::Display;

#[derive(Debug)]
pub enum AWSIoTError {
    /// Error establishing or maintaining the MQTT connection.
    AWSConnectionError(Box<ConnectionError>),
    /// Error performing an MQTT client operation (subscribe, publish, etc).
    MQTTClientError(ClientError),
    /// Error reading certificate or key files.
    IoError(std::io::Error),
    /// A mutex was poisoned (sync client only).
    MutexError(String),
}

impl Display for AWSIoTError {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        match self {
            AWSIoTError::AWSConnectionError(err) => {
                write!(f, "Problem connecting to AWS: {err}")
            }
            AWSIoTError::MQTTClientError(err) => {
                write!(f, "MQTT client error: {err}")
            }
            AWSIoTError::IoError(err) => write!(f, "Problem reading file: {err}"),
            AWSIoTError::MutexError(msg) => write!(f, "Mutex poisoned: {msg}"),
        }
    }
}

impl std::error::Error for AWSIoTError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AWSIoTError::AWSConnectionError(err) => Some(err),
            AWSIoTError::MQTTClientError(err) => Some(err),
            AWSIoTError::IoError(err) => Some(err),
            AWSIoTError::MutexError(_) => None,
        }
    }
}

impl From<std::io::Error> for AWSIoTError {
    fn from(err: std::io::Error) -> AWSIoTError {
        AWSIoTError::IoError(err)
    }
}

impl From<ConnectionError> for AWSIoTError {
    fn from(err: ConnectionError) -> AWSIoTError {
        AWSIoTError::AWSConnectionError(Box::new(err))
    }
}

impl From<ClientError> for AWSIoTError {
    fn from(err: ClientError) -> AWSIoTError {
        AWSIoTError::MQTTClientError(err)
    }
}

impl<T> From<std::sync::PoisonError<T>> for AWSIoTError {
    fn from(err: std::sync::PoisonError<T>) -> AWSIoTError {
        AWSIoTError::MutexError(err.to_string())
    }
}
