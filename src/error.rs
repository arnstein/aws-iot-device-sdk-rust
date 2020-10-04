use std::fmt::Display;
use std::fmt;
use rumqttc::ConnectionError;
use async_channel::{RecvError as AsyncRecvError, SendError as AsyncSendError};

#[derive(Debug)]
pub enum AWSIoTError {
    AsyncChannelReceiveError,
    AsyncChannelSendError,
    AWSConnectionError,
}

impl Display for AWSIoTError {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        match self {
            AWSIoTError::AWSConnectionError => write!(f, "Problem connecting to AWS"),
            AWSIoTError::AsyncChannelReceiveError => write!(f, "Problem receiving on internal channel"),
            AWSIoTError::AsyncChannelSendError => write!(f, "Problem sending on internal channel"),
        }
    }
}

impl From<AsyncRecvError> for AWSIoTError {
    fn from(_err: AsyncRecvError) -> AWSIoTError {
        AWSIoTError::AsyncChannelReceiveError
    }
}

impl<T> From<AsyncSendError<T>> for AWSIoTError {
    fn from(_err: AsyncSendError<T>) -> AWSIoTError {
        AWSIoTError::AsyncChannelSendError
    }
}

impl From<ConnectionError> for AWSIoTError {
    fn from(_err: ConnectionError) -> AWSIoTError {
        AWSIoTError::AWSConnectionError
    }
}
