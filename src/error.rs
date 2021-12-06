use std::fmt::Display;
use std::fmt;
use rumqttc::ConnectionError;

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
