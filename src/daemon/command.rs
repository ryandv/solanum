extern crate regex;

use daemon::chrono::Duration;
use daemon::chrono::datetime::DateTime;
use daemon::chrono::offset::utc::UTC;
use daemon::result::Error;
use daemon::result::Result;

use std::fmt::Display;
use std::fmt::Error as FmtError;
use std::fmt::Formatter;
use std::iter::FromIterator;
use std::result::Result as StdResult;

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Start(DateTime<UTC>, Duration, Duration, Vec<String>),
    Stop,
    List,
    Status,
}

impl Display for Command {
    fn fmt(&self, f: &mut Formatter) -> StdResult<(), FmtError> {
        match *self {
            Command::Start(start_time, work_duration, break_duration, ref tags) => {
                let tags_csv = tags.into_iter().fold(String::from(""), |acc, tag| format!("{}{},", acc, tag));

                write!(f, "START: {} {} {} {}", start_time, work_duration, break_duration, tags_csv)
            }
            Command::Stop => write!(f, "STOP"),
            Command::List => write!(f, "LIST"),
            Command::Status => write!(f, "STATUS")
        }
    }
}

impl Command {
    pub fn from_string(current_time: DateTime<UTC>,
                       string: String)
                       -> Result<Command> {
        let start_re = regex::Regex::new(r"^START(?: tags ((?:\w+,)*(?:\w+)))?(?: (\d+) (\d+))?")
            .unwrap();
        if start_re.is_match(string.as_str()) {
            match start_re.captures(string.as_str()) {
                Some(caps) => {
                    let tags_csv: String =
                        caps.at(1).unwrap_or("").parse().unwrap_or(String::from(""));
                    let tags = Vec::from_iter(tags_csv
                                              .split(",")
                                              .filter(|tag| !tag.is_empty())
                                              .map(|s| String::from(s))
                                             );

                    let work_time_argument: i64 =
                        caps.at(2).unwrap_or("1500").parse().unwrap_or(1500);
                    let break_time_argument: i64 =
                        caps.at(3).unwrap_or("300").parse().unwrap_or(300);
                    let work_time = Duration::seconds(work_time_argument);
                    let break_time = Duration::seconds(break_time_argument);

                    Ok(Command::Start(current_time, work_time, break_time, tags))
                }
                None => {
                    Ok(Command::Start(current_time,
                                      Duration::seconds(1500),
                                      Duration::seconds(300),
                                      vec![]))
                }
            }
        } else if string == "STOP" {
            Ok(Command::Stop)
        } else if string == "LIST" {
            Ok(Command::List)
        } else if string == "STATUS" {
            Ok(Command::Status)
        } else {
            Err(Error::from(format!("Invalid command string: {}", string)))
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
                               Duration::seconds(300),
                               vec![])
                );
    }

    #[test]
    fn can_parse_start_commands_with_custom_durations() {
        let current_time = "2000-01-01T00:00:00+00:00".parse::<DateTime<UTC>>().unwrap();
        let string = String::from("START 23 42");

        let command = Command::from_string(current_time, string);

        assert!(command.unwrap() ==
                Command::Start(current_time, Duration::seconds(23), Duration::seconds(42), vec![]));
    }

    #[test]
    fn can_parse_start_commands_with_tags() {
        let current_time = "2000-01-01T00:00:00+00:00".parse::<DateTime<UTC>>().unwrap();
        let string = String::from("START tags foo,bar,baz");

        let command = Command::from_string(current_time, string);

        assert!(command.unwrap() ==
                Command::Start(
                    current_time,
                    Duration::seconds(1500),
                    Duration::seconds(300),
                    vec![String::from("foo"), String::from("bar"), String::from("baz")])
                );
    }

    #[test]
    fn can_parse_start_commands_with_tags_and_custom_durations() {
        let current_time = "2000-01-01T00:00:00+00:00".parse::<DateTime<UTC>>().unwrap();
        let string = String::from("START tags foo,bar,baz 23 42");

        let command = Command::from_string(current_time, string);

        assert!(command.unwrap() ==
                Command::Start(
                    current_time,
                    Duration::seconds(23),
                    Duration::seconds(42),
                    vec![String::from("foo"), String::from("bar"), String::from("baz")])
                );
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
