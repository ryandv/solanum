extern crate mio;

use daemon::CanHandle;

use std::io;
use std::result;

pub struct SignalEventSubscriber<'a> {
    io: mio::unix::EventedFd<'a>,
    token: mio::Token
}

impl<'a> SignalEventSubscriber<'a> {
    pub fn new(io: mio::unix::EventedFd, token: mio::Token) -> SignalEventSubscriber {
        SignalEventSubscriber {
            io: io,
            token: token
        }
    }
}

impl<'a> CanHandle for SignalEventSubscriber<'a> {
    fn handle(&self) -> result::Result<io::Result<()>, io::Result<()>> {
        info!("Signal received");
        Err(Ok(()))
    }

    fn token(&self) -> mio::Token {
        self.token
    }

    fn io(&self) -> &mio::Evented {
        &self.io
    }

}
