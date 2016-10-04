extern crate mio;
extern crate mio_uds;
extern crate regex;
extern crate time;

use self::mio::{ Poll, PollOpt, Ready, Token };
use self::mio_uds::UnixListener;

use std::fs;
use std::iter::FromIterator;
use std::io::{ Error, ErrorKind, Read, Write };
use std::net::Shutdown;
use std::path::Path;
use std::vec::Vec;

enum Command {
    Start(time::Tm, time::Duration, time::Duration),
    Stop
}

pub struct CommandProcessor {
    listener : UnixListener,
    responder : CommandResponder
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

impl CommandProcessor {
    pub fn new(poll : &Poll) -> Result<CommandProcessor, Error>
    {
        let listener = try!(UnixListener::bind("/tmp/solanum"));
        try!(poll.register(&listener, Token(0), Ready::readable(), PollOpt::edge()));

        let responder = CommandResponder::new();

        Ok(CommandProcessor {
            listener : listener,
            responder : responder
        })
    }

    pub fn handle_acceptor(&self) -> Result<(), Error>
    {
        let accept_option = try!(self.listener.accept());
        match accept_option {
            Some((mut stream, _)) => {
                let mut buf : [u8; 1024] = [0; 1024];
                try!(stream.read(&mut buf));
                let codepoints = Vec::from_iter(buf.to_vec().into_iter().take_while(|codepoint| *codepoint != (0 as u8)));
                let message = String::from_utf8(codepoints).unwrap();
                let command = try!(Command::from_string(message));

                try!(stream.write_all(self.responder.respond(command).as_bytes()));

                try!(stream.shutdown(Shutdown::Both));
                Ok(())
            },
            None => {
                //log: tried to accept but no connection from other end
                Ok(())
            }
        }
    }
}

impl Drop for CommandProcessor {
    fn drop(&mut self)
    {
        // TODO: log errors instead of just silently discarding.
        // right now, silently discarding errors to ensure listener is recursively dropped.
        match fs::remove_file(Path::new("/tmp/solanum")) {
            Ok(_) => {},
            Err(_) => {}
        }
    }
}

struct CommandResponder {
}

impl CommandResponder {
    pub fn new() -> CommandResponder {
        CommandResponder {}
    }

    pub fn respond(&self, command : Command) -> String {
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

    use super::Command;
    use super::CommandResponder;
    use super::time;

    #[test]
    fn responds_to_start_commands_with_the_current_time()
    {
        let responder = CommandResponder::new();

        let response = responder.respond(Command::Start(time::strptime("2020-01-01 00:00:00", "%F %H:%M:%S").unwrap(), time::Duration::seconds(42), time::Duration::seconds(42)));

        assert!(response == "Pomodoro started at 2020-01-01 00:00:00");
    }

    #[test]
    fn responds_to_stop_commands()
    {
        let responder = CommandResponder::new();

        let response = responder.respond(Command::Stop);

        assert!(response.contains("Pomodoro aborted"));
    }
}
