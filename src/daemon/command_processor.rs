extern crate mio;
extern crate mio_uds;
extern crate time;

use self::mio::{ Poll, PollOpt, Ready, Token };
use self::mio_uds::UnixListener;

use std::fs;
use std::io::{ Error, Read, Write };
use std::net::Shutdown;
use std::path::Path;

pub struct CommandProcessor
{
    listener : UnixListener
}

impl CommandProcessor
{
    pub fn new(poll : &Poll) -> Result<CommandProcessor, Error>
    {
        let listener = try!(UnixListener::bind("/tmp/solanum"));
        try!(poll.register(&listener, Token(0), Ready::readable(), PollOpt::edge()));

        Ok(CommandProcessor { listener : listener })
    }

    pub fn handle_acceptor(&self) -> Result<(), Error>
    {
        let accept_option = try!(self.listener.accept());
        match accept_option {
            Some((mut stream, _)) => {
                let mut buf : [u8; 1024] = [0; 1024];

                try!(stream.read(&mut buf));
                let command = String::from_utf8(buf.to_vec()).unwrap();

                let current_time = time::strftime("%F %H:%M:%S", &time::now()).unwrap();
                try!(stream.write_all(format!("Pomodoro started at {}", current_time).as_bytes()));

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

impl Drop for CommandProcessor
{
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
