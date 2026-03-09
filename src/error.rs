use rumqttc::ConnectionError;
use std::fmt;
use std::fmt::Display;

#[derive(Debug)]
pub enum AWSIoTError {
    AWSConnectionError(Box<ConnectionError>),
    IoError(std::io::Error),
}

impl Display for AWSIoTError {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        match self {
            AWSIoTError::AWSConnectionError(err) => {
                write!(f, "Problem connecting to AWS: {err}")
            }
            AWSIoTError::IoError(err) => write!(f, "Problem reading file: {err}"),
        }
    }
}

impl std::error::Error for AWSIoTError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AWSIoTError::AWSConnectionError(err) => Some(err),
            AWSIoTError::IoError(err) => Some(err),
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
