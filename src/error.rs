use rumqttc::ConnectionError;
use std::fmt;
use std::fmt::Display;

#[derive(Debug)]
pub enum AWSIoTError {
    AWSConnectionError,
    IoError,
}

impl Display for AWSIoTError {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        match self {
            AWSIoTError::AWSConnectionError => write!(f, "Problem connecting to AWS"),
            AWSIoTError::IoError => write!(f, "Problem reading file"),
        }
    }
}

impl std::error::Error for AWSIoTError {}

impl From<std::io::Error> for AWSIoTError {
    fn from(_err: std::io::Error) -> AWSIoTError {
        AWSIoTError::IoError
    }
}

impl From<ConnectionError> for AWSIoTError {
    fn from(_err: ConnectionError) -> AWSIoTError {
        AWSIoTError::AWSConnectionError
    }
}
