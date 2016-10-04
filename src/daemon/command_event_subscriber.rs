extern crate mio;

use daemon::{ CanHandle, CommandProcessor };

use std::error::Error;
use std::io;
use std::result;

pub struct CommandEventSubscriber<'a> {
    io: &'a CommandProcessor,
    token: mio::Token
}

impl<'a> CommandEventSubscriber<'a> {
    pub fn new(io: &'a CommandProcessor, token: mio::Token) -> CommandEventSubscriber<'a> {
        CommandEventSubscriber {
            io: io,
            token: token
        }
    }
}

impl<'a> CanHandle for CommandEventSubscriber<'a> {
    fn handle(&self) -> result::Result<io::Result<()>, io::Result<()>> {
        match self.io.handle_acceptor() {
            Ok(_) => { info!("Handled command"); Ok(Ok(())) }
            Err(e) => {
                error!("{}", e.description());
                Err(Err(e))
            }
        }
    }

    fn token(&self) -> mio::Token {
        self.token
    }

    fn io(&self) -> &mio::Evented {
        self.io
    }
}
