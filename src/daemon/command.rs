extern crate chrono;
extern crate regex;

use self::chrono::offset::utc::UTC;

use std::io::{ Error, ErrorKind };

pub enum Command {
    Start(chrono::datetime::DateTime<UTC>, chrono::Duration, chrono::Duration),
    Stop,
    List
}

impl Command {
    pub fn from_string(string : String) -> Result<Command, Error> {
        let start_re = regex::Regex::new(r"^START(?: ((?:\w+,)*(?:\w+)) )?( ?:(\d+) (\d+))?").unwrap();
        if start_re.is_match(string.as_str()) {
            match start_re.captures(string.as_str()) {
                Some(caps) => {
                    let work_time_argument : i64 = caps.at(2).unwrap_or("1500").parse().unwrap_or(1500);
                    let break_time_argument : i64 = caps.at(3).unwrap_or("300").parse().unwrap_or(300);
                    let work_time = chrono::Duration::seconds(work_time_argument);
                    let break_time = chrono::Duration::seconds(break_time_argument);
                    Ok(Command::Start(UTC::now(), work_time, break_time))
                },
                None => {
                    Ok(Command::Start(UTC::now(), chrono::Duration::seconds(1500), chrono::Duration::seconds(300)))
                }
            }
        } else if string == "STOP" {
            Ok(Command::Stop)
        } else if string == "LIST" {
            Ok(Command::List)
        } else {
            Err(Error::new(ErrorKind::InvalidInput, format!("command not recognized: {}", string)))
        }
    }
}
