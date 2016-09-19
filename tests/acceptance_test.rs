extern crate solanum;
extern crate nix;

#[cfg(test)]
mod spec {
    use solanum::client;

    use std::process::Command;
    use std::error::Error;
    use std::path::Path;
    use std::result;

    use nix::libc::pid_t;
    use nix::sys::signal;
    use nix::unistd::{sleep};

    #[test]
    fn client_can_communicate_with_daemon() {
        Command::new("target/debug/solanumd").spawn();
        sleep(1);
        let client = client::Client {};
        let result = client.send_message();
        assert!(result.is_ok());
    }

    #[test]
    fn client_returns_error_when_daemon_is_not_active() {
        let client = client::Client {};
        let response = client.send_message();
        assert!(response.is_err());
    }

    #[test]
    #[ignore]
    fn daemon_closes_listener_socket_on_sigterm() {
        let daemon = Command::new("target/debug/solanumd").spawn().unwrap();
        let socket_path = Path::new("/tmp/solanum");
        sleep(1);
        signal::kill(daemon.id() as pid_t, signal::Signal::SIGTERM);
        sleep(3);
        assert!(!socket_path.exists());
    }
}
