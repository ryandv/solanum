extern crate mio;
extern crate mio_uds;
extern crate regex;
extern crate time;

use daemon::Command;
use daemon::CommandResponder;

use self::mio::{ Evented, Poll, PollOpt, Ready, Token };
use self::mio_uds::UnixListener;

use std::fs;
use std::iter::FromIterator;
use std::io;
use std::io::{ Read, Write };
use std::net::Shutdown;
use std::path::Path;
use std::vec::Vec;

pub struct CommandProcessor {
    responder : CommandResponder
}

impl CommandProcessor {
    pub fn new() -> io::Result<CommandProcessor>
    {

        let responder = CommandResponder::new();

        Ok(CommandProcessor {
            responder : responder
        })
    }

    pub fn handle_acceptor(&self, command: Command) -> String
    {
        self.responder.respond(command)
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
