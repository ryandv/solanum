extern crate postgres;
extern crate mio;
extern crate mio_uds;
extern crate regex;
extern crate time;

use daemon::Command;
use daemon::PomodoroQueryMapper;

use self::mio::{ Evented, Poll, PollOpt, Ready, Token };
use self::mio_uds::UnixListener;

use std::io;
use std::string;

pub struct CommandProcessor {
}

impl CommandProcessor {
    pub fn new() -> CommandProcessor
    {
        CommandProcessor {
        }
    }

    pub fn handle_command(&self, command: Command) -> String
    {
        match command {
            Command::Start(start_time, work_duration, break_duration) => self.handle_start(&start_time, work_duration, break_duration),
            Command::Stop => self.handle_stop(),
            Command::List => self.handle_list()
        }
    }

    fn handle_start(&self, start_time : &time::Tm, work_duration: time::Duration, break_duration: time::Duration) -> String {
        let new_start_time = PomodoroQueryMapper::create_pomodoro(start_time, work_duration, break_duration).
            and_then(|_| PomodoroQueryMapper::get_most_recent_pomodoro_start_time());

        match new_start_time {
            Ok(time) => format!("Pomodoro started at {}", time::strftime("%F %H:%M:%S", &time).unwrap()),
            Err(_) => format!("Failed to start pomodoro.")
        }
    }

    fn handle_stop(&self) -> String {
        let conn = postgres::Connection::connect("postgres://postgres@localhost:5432/solanum_test", postgres::SslMode::None).unwrap();
        let rows = &conn.
            query("SELECT id, work_start_time, work_end_time, break_start_time, break_end_time, work_length, break_length, status FROM pomodoros ORDER BY work_start_time DESC LIMIT 1", &[]).
            unwrap();

        let last_pomodoro = rows.get(0);

        let id: i32 = last_pomodoro.get(0);
        let work_start_time: time::Timespec = last_pomodoro.get(1);
        let work_end_time: Option<time::Timespec> = last_pomodoro.get(2);
        let break_start_time: Option<time::Timespec> = last_pomodoro.get(3);
        let break_end_time: Option<time::Timespec> = last_pomodoro.get(4);
        let work_length: i64 = last_pomodoro.get(5);
        let break_length: i64 = last_pomodoro.get(6);
        let status: String = last_pomodoro.get(7);

        &conn.execute(
            "UPDATE pomodoros SET status = $2
            WHERE id = $1",
                &[&id, &String::from("COMPLETED")]
                ).unwrap();

        String::from("Pomodoro aborted")
    }

    fn handle_list(&self) -> String {
        let conn = postgres::Connection::connect("postgres://postgres@localhost:5432/solanum_test", postgres::SslMode::None).unwrap();
        let response = (&conn).
            query("SELECT work_start_time, status, tags FROM pomodoros ORDER BY work_start_time DESC LIMIT 5", &[]).
            unwrap().
            into_iter().
            fold(String::from(""), |acc, pomodoro| {
                let start_time: time::Timespec = pomodoro.get(0);
                let status: String = pomodoro.get(1);
                let tags: String = pomodoro.get(2);
                acc + &format!("[{}]: {} ({})\n", time::strftime("%F %H:%M:%S", &time::at(start_time)).unwrap(), status, tags)
            });
        response
    }
}

#[cfg(test)]
mod test {
    use daemon::Command;

    use super::CommandProcessor;
    use super::time;

    // IGNORED pending resolution of test db teardown
    #[ignore]
    #[test]
    fn responds_to_start_commands_with_the_current_time()
    {
        let processor = CommandProcessor::new();

        let response = processor.handle_command(Command::Start(time::strptime("2000-01-01 00:00:00", "%F %H:%M:%S").unwrap(), time::Duration::seconds(42), time::Duration::seconds(42)));
        println!("&UHJNM {}", response);

        assert!(response == "Pomodoro started at 2000-01-01 00:00:00");
    }

    #[test]
    fn responds_to_stop_commands()
    {
        let processor = CommandProcessor::new();

        let response = processor.handle_command(Command::Stop);

        assert!(response.contains("Pomodoro aborted"));
    }
}
