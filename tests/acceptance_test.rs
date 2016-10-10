#[cfg(test)]
mod spec {
    extern crate chrono;
    extern crate solanum;
    extern crate nix;

    use self::solanum::client;

    use std::process;
    use std::fs;
    use std::io::Read;
    use std::path::Path;

    use self::nix::libc::pid_t;
    use self::nix::sys::signal;
    use self::nix::unistd::{sleep};

    #[test]
    fn full_lifecycle_test() {
        let client = client::Client::new();
        client_returns_error_when_daemon_is_not_active(&client);
        client_can_start_a_pomodoro(&client);
        client_can_abort_a_pomodoro(&client);
        client_can_complete_a_pomodoro_work_period(&client);
        daemon_closes_listener_socket_on_sigterm();
    }

    fn client_returns_error_when_daemon_is_not_active(client : &client::Client) {
        let response = client.send_message(String::from("START"));
        assert!(response.is_err());
    }

    fn client_can_start_a_pomodoro(client : &client::Client) {
        process::Command::new("target/debug/solanumd").spawn().unwrap();
        sleep(1);

        let result = client.send_message(String::from("START"));

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(pomodoro_is_started_at_current_time(response));
    }

    fn client_can_abort_a_pomodoro(client : &client::Client) {
        let result = client.send_message(String::from("STOP"));

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(pomodoro_is_aborted(response));
    }

    fn client_can_complete_a_pomodoro_work_period(client : &client::Client) {
        let start_response = client.send_message(String::from("START 1 1")).unwrap();
        sleep(1);
        client.send_message(String::from("STOP")).unwrap();

        let list_response = client.send_message(String::from("LIST")).unwrap();

        assert!(list_response.contains("BreakPending"));
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
        let expected_response = format!("Pomodoro started at {}", chrono::offset::utc::UTC::now().format("%F %H:%M:%S").to_string());
        // trim off seconds to allow some tolerance
        println!("{}", response);
        let expected_response_without_seconds = &expected_response[0..expected_response.len()-3];
        response.contains(expected_response_without_seconds)
    }

    fn pomodoro_is_aborted(response : String) -> bool {
        response.contains("Pomodoro aborted")
    }
}
