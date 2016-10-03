extern crate mio;
extern crate mio_uds;

use self::mio::{ Poll, PollOpt, Ready, Token };
use self::mio_uds::UnixListener;

use std::fs;
use std::io::{ Error, Read };
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
                let mut message = String::new();
                try!(stream.read_to_string(&mut message));
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
        fs::remove_file(Path::new("/tmp/solanum")).unwrap();
    }
}
