use aws_iot_device_sdk_rust::error::AWSIoTError;
use std::error::Error;
use std::io;

#[test]
fn io_error_preserves_source() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let aws_err: AWSIoTError = io_err.into();

    let display = format!("{aws_err}");
    assert!(display.contains("file not found"));
    assert!(aws_err.source().is_some());
}

#[test]
fn io_error_display() {
    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
    let aws_err: AWSIoTError = io_err.into();
    let msg = format!("{aws_err}");
    assert!(msg.contains("Problem reading file"));
    assert!(msg.contains("access denied"));
}

#[test]
fn error_is_debug() {
    let io_err = io::Error::new(io::ErrorKind::Other, "test");
    let aws_err: AWSIoTError = io_err.into();
    let debug = format!("{aws_err:?}");
    assert!(debug.contains("IoError"));
}
