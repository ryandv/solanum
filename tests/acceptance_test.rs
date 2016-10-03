extern crate solanum;
extern crate nix;

#[cfg(test)]
mod spec {
    use solanum::client;

    use std::process;
    use std::fs;
    use std::io::Read;
    use std::error::Error;
    use std::path::Path;
    use std::result;

    use nix::libc::pid_t;
    use nix::sys::signal;
    use nix::unistd::{sleep};

    #[test]
    fn full_lifecycle_test() {
        client_returns_error_when_daemon_is_not_active();
        client_can_communicate_with_daemon();
        daemon_closes_listener_socket_on_sigterm();
    }

    fn client_returns_error_when_daemon_is_not_active() {
        let client = client::Client {};
        let response = client.send_message();
        assert!(response.is_err());
    }

    fn client_can_communicate_with_daemon() {
        process::Command::new("target/debug/solanumd").spawn().unwrap();
        sleep(1);
        let client = client::Client {};
        let result = client.send_message();
        assert!(result.is_ok());
    }

    fn daemon_closes_listener_socket_on_sigterm() {
        let socket_path = Path::new("/tmp/solanum");
        let pidfile_path = Path::new("/tmp/solanum.pid");
        let mut pidfile = fs::File::open(pidfile_path).unwrap();
        let mut pidString = String::new();
        pidfile.read_to_string(&mut pidString).unwrap();
        signal::kill(pidString.parse::<pid_t>().unwrap() as pid_t, signal::Signal::SIGTERM).unwrap();
        sleep(1);
        assert!(!socket_path.exists());
    }
}
