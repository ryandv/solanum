use daemon::io::{CanHandle, CanSend};
use daemon::result::Result;

use std::io;

use super::mio;

pub struct SignalEventSubscriber<'a> {
    io: mio::unix::EventedFd<'a>,
    token: mio::Token,
}

impl<'a> SignalEventSubscriber<'a> {
    pub fn new(io: mio::unix::EventedFd, token: mio::Token) -> SignalEventSubscriber {
        SignalEventSubscriber {
            io: io,
            token: token,
        }
    }
}

impl<'a, S: CanSend<bool>> CanHandle<'a, S> for SignalEventSubscriber<'a> {
    fn handle(&self, stop_sender: S) -> Result<()> {
        info!("Signal received");
        stop_sender.send(true)
    }

    fn token(&self) -> mio::Token {
        self.token
    }

    fn io(&self) -> &mio::Evented {
        &self.io
    }
}
