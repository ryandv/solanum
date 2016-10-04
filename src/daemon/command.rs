extern crate regex;
extern crate time;

use std::io::{ Error, ErrorKind };

pub enum Command {
    Start(time::Tm, time::Duration, time::Duration),
    Stop
}

impl Command {
    pub fn from_string(string : String) -> Result<Command, Error> {
        let start_re = regex::Regex::new(r"^START(?: ((?:\w+,)*(?:\w+)) )?( ?:(\d+) (\d+))?").unwrap();
        if start_re.is_match(string.as_str()) {
            match start_re.captures(string.as_str()) {
                Some(caps) => {
                    let work_time_argument : i64 = caps.at(2).unwrap_or("1500").parse().unwrap_or(1500);
                    let break_time_argument : i64 = caps.at(3).unwrap_or("300").parse().unwrap_or(300);
                    let work_time = time::Duration::seconds(work_time_argument);
                    let break_time = time::Duration::seconds(break_time_argument);
                    Ok(Command::Start(time::now(), work_time, break_time))
                },
                None => {
                    Ok(Command::Start(time::now(), time::Duration::seconds(1500), time::Duration::seconds(300)))
                }
            }
        } else if string == "STOP" {
            Ok(Command::Stop)
        } else {
            Err(Error::new(ErrorKind::InvalidInput, format!("command not recognized: {}", string)))
        }
    }
}
