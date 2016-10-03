extern crate solanum;
extern crate time;
extern crate nix;

#[cfg(test)]
mod spec {
    use solanum::client;

    use std::process;
    use std::fs;
    use std::io::Read;
    use std::path::Path;

    use time;

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
        let response = client.send_message(String::from("START"));
        assert!(response.is_err());
    }

    fn client_can_communicate_with_daemon() {
        process::Command::new("target/debug/solanumd").spawn().unwrap();
        sleep(1);
        let client = client::Client {};
        let result = client.send_message(String::from("START"));
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(pomodoro_is_started_at_current_time(response));
    }

    fn daemon_closes_listener_socket_on_sigterm() {
        let socket_path = Path::new("/tmp/solanum");
        let pidfile_path = Path::new("/tmp/solanum.pid");
        let mut pidfile = fs::File::open(pidfile_path).unwrap();
        let mut pidstring = String::new();
        pidfile.read_to_string(&mut pidstring).unwrap();
        signal::kill(pidstring.parse::<pid_t>().unwrap() as pid_t, signal::Signal::SIGTERM).unwrap();
        sleep(1);
        assert!(!socket_path.exists());
        assert!(!pidfile_path.exists());
    }

    fn pomodoro_is_started_at_current_time(response : String) -> bool {
        let expected_response = format!("Pomodoro started at {}", time::strftime("%F %H:%M:%S", &time::now()).unwrap());
        // trim off seconds to allow some tolerance
        let expected_response_without_seconds = &expected_response[0..expected_response.len()-3];
        response.contains(expected_response_without_seconds)
    }
}
