extern crate mio;
extern crate mio_uds;
extern crate regex;
extern crate time;

use daemon::Command;
use daemon::CommandResponder;

use self::mio::{ Poll, PollOpt, Ready, Token };
use self::mio_uds::UnixListener;

use std::fs;
use std::iter::FromIterator;
use std::io::{ Error, Read, Write };
use std::net::Shutdown;
use std::path::Path;
use std::vec::Vec;

pub struct CommandProcessor {
    listener : UnixListener,
    responder : CommandResponder
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
