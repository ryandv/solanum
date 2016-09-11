extern crate solanum;

#[cfg(test)]
mod spec {
    use solanum::daemon;
    use solanum::client;

    use std::result;

    #[test]
    fn client_can_communicate_with_daemon() {
        let daemon = daemon::Daemon {};
        let client = client::Client {};
        let response = client.send_message();
        assert!(response.is_ok());
    }
}
