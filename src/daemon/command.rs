extern crate regex;

use daemon::chrono::Duration;
use daemon::chrono::datetime::DateTime;
use daemon::chrono::offset::utc::UTC;

use std::fmt::Display;
use std::fmt::Error as FmtError;
use std::fmt::Formatter;
use std::error::Error;

#[derive(PartialEq, Eq)]
pub enum Command {
    Start(DateTime<UTC>, Duration, Duration),
    Stop,
    List,
    Status,
}

#[derive(Debug)]
pub struct InvalidCommandString {
    command_string: String,
}

impl InvalidCommandString {
    pub fn new(command_string: String) -> InvalidCommandString {
        InvalidCommandString { command_string: format!("Invalid command: {}", command_string) }
    }
}

impl Display for InvalidCommandString {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "{}", self.command_string)
    }
}

impl Error for InvalidCommandString {
    fn description(&self) -> &str {
        &self.command_string
    }
}

impl Command {
    pub fn from_string(current_time: DateTime<UTC>,
                       string: String)
                       -> Result<Command, InvalidCommandString> {
        let start_re = regex::Regex::new(r"^START(?: tags ((?:\w+,)*(?:\w+)))?(?: (\d+) (\d+))?")
            .unwrap();
        if start_re.is_match(string.as_str()) {
            match start_re.captures(string.as_str()) {
                Some(caps) => {
                    let work_time_argument: i64 =
                        caps.at(2).unwrap_or("1500").parse().unwrap_or(1500);
                    let break_time_argument: i64 =
                        caps.at(3).unwrap_or("300").parse().unwrap_or(300);
                    let work_time = Duration::seconds(work_time_argument);
                    let break_time = Duration::seconds(break_time_argument);
                    Ok(Command::Start(current_time, work_time, break_time))
                }
                None => {
                    Ok(Command::Start(current_time,
                                      Duration::seconds(1500),
                                      Duration::seconds(300)))
                }
            }
        } else if string == "STOP" {
            Ok(Command::Stop)
        } else if string == "LIST" {
            Ok(Command::List)
        } else if string == "STATUS" {
            Ok(Command::Status)
        } else {
            Err(InvalidCommandString::new(string))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_parse_start_commands_with_default_durations() {
        let current_time = "2000-01-01T00:00:00+00:00".parse::<DateTime<UTC>>().unwrap();
        let string = String::from("START");

        let command = Command::from_string(current_time, string);

        assert!(command.unwrap() ==
                Command::Start(current_time,
                               Duration::seconds(1500),
                               Duration::seconds(300)));
    }

    #[test]
    fn can_parse_start_commands_with_custom_durations() {
        let current_time = "2000-01-01T00:00:00+00:00".parse::<DateTime<UTC>>().unwrap();
        let string = String::from("START 23 42");

        let command = Command::from_string(current_time, string);

        assert!(command.unwrap() ==
                Command::Start(current_time, Duration::seconds(23), Duration::seconds(42)));
    }

    #[test]
    fn can_parse_status_commands() {
        let current_time = "2000-01-01T00:00:00+00:00".parse::<DateTime<UTC>>().unwrap();
        let string = String::from("STATUS");

        let command = Command::from_string(current_time, string);

        assert!(command.unwrap() == Command::Status);
    }

    #[test]
    fn returns_error_when_given_invalid_string() {
        let current_time = "2000-01-01T00:00:00+00:00".parse::<DateTime<UTC>>().unwrap();
        let string = String::from("FOOBAR");

        let command = Command::from_string(current_time, string);

        assert!(command.is_err())
    }
}
