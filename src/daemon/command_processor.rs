extern crate mio;
extern crate mio_uds;
extern crate regex;
extern crate time;

use daemon::Command;

use self::mio::{ Evented, Poll, PollOpt, Ready, Token };
use self::mio_uds::UnixListener;

use std::io;

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
            Command::Start(start_time, _, _) => self.handle_start(&start_time),
            Command::Stop => self.handle_stop()
        }
    }

    fn handle_start(&self, start_time : &time::Tm) -> String {
        format!("Pomodoro started at {}", time::strftime("%F %H:%M:%S", &start_time).unwrap())
    }

    fn handle_stop(&self) -> String {
        String::from("Pomodoro aborted")
    }
}

#[cfg(test)]
mod test {
    use daemon::Command;

    use super::CommandProcessor;
    use super::time;

    #[test]
    fn responds_to_start_commands_with_the_current_time()
    {
        let processor = CommandProcessor::new();

        let response = processor.handle_command(Command::Start(time::strptime("2020-01-01 00:00:00", "%F %H:%M:%S").unwrap(), time::Duration::seconds(42), time::Duration::seconds(42)));

        assert!(response == "Pomodoro started at 2020-01-01 00:00:00");
    }

    #[test]
    fn responds_to_stop_commands()
    {
        let processor = CommandProcessor::new();

        let response = processor.handle_command(Command::Stop);

        assert!(response.contains("Pomodoro aborted"));
    }
}
