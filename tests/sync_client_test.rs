#[cfg(feature = "sync")]
mod sync_tests {
    use aws_iot_device_sdk_rust::AWSIoTSettings;

    fn make_settings() -> AWSIoTSettings {
        AWSIoTSettings::new(
            "test-client".to_owned(),
            "nonexistent_ca.pem".to_owned(),
            "nonexistent_cert.crt".to_owned(),
            "nonexistent_key.pem".to_owned(),
            "endpoint.amazonaws.com".to_owned(),
            None,
        )
    }

    #[test]
    fn new_client_fails_with_missing_certs() {
        use aws_iot_device_sdk_rust::AWSIoTClient;

        let settings = make_settings();
        let result = AWSIoTClient::new(settings);
        match result {
            Err(err) => {
                let msg = format!("{err}");
                assert!(msg.contains("Problem reading file"));
            }
            Ok(_) => panic!("Should fail when cert files don't exist"),
        }
    }
}
