extern crate chrono;

use self::chrono::datetime::DateTime;
use self::chrono::offset::utc::UTC;

use daemon::Command;
use daemon::PomodoroQueryMapper;

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

    fn handle_start(&self, start_time : &DateTime<UTC>, work_duration: chrono::Duration, break_duration: chrono::Duration) -> String {
        let new_pomodoro = PomodoroQueryMapper::create_pomodoro(start_time, work_duration, break_duration).
            and_then(|_| PomodoroQueryMapper::get_most_recent_pomodoro());

        match new_pomodoro {
            Ok(pomodoro) => format!("Pomodoro started at {}", pomodoro.work_start_time.format("%F %H:%M:%S").to_string()),
            Err(_) => format!("Failed to start pomodoro.")
        }
    }

    fn handle_stop(&self) -> String {
        let result = PomodoroQueryMapper::get_most_recent_pomodoro().
            and_then(|pomodoro| PomodoroQueryMapper::stop_pomodoro(pomodoro.id) );

        match result {
            Ok(_) => String::from("Pomodoro aborted"),
            Err(_) => String::from("Failed to abort pomodoro")
        }
    }

    fn handle_list(&self) -> String {
        let last_five_pomodoros = PomodoroQueryMapper::list_most_recent_pomodoros(5).
            and_then(|pomodoros| Ok(
                    pomodoros.into_iter().fold(String::from(""), |acc, pomodoro| {
                        acc + &format!("[{}]: {} ({})\n", pomodoro.work_start_time.format("%F %H:%M:%S").to_string(), pomodoro.status, pomodoro.tags)
                    })));

        match last_five_pomodoros {
            Ok(pomodoro_descriptions) => pomodoro_descriptions,
            Err(_) => String::from("Unable to list pomodoros")
        }
    }
}

#[cfg(test)]
mod test {
    use daemon::Command;

    use super::CommandProcessor;
    use super::chrono::Duration;
    use super::chrono::TimeZone;
    use super::chrono::datetime::DateTime;
    use super::chrono::offset::utc::UTC;

    // IGNORED pending resolution of test db teardown
    #[ignore]
    #[test]
    fn responds_to_start_commands_with_the_current_time()
    {
        let processor = CommandProcessor::new();

        let response = processor.handle_command(Command::Start("2000-01-01T00:00:00+00:00".parse::<DateTime<UTC>>().unwrap(), Duration::seconds(42), Duration::seconds(42)));
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
