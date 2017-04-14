use daemon::io::{EventSubscriber, CanSend};
use daemon::result::Result;

use super::mio;

pub struct SignalEventSubscriber<'a> {
    io: mio::unix::EventedFd<'a>,
    token: mio::Token,
}

unsafe impl<'a> Sync for SignalEventSubscriber<'a> { }

impl<'a> SignalEventSubscriber<'a> {
    pub fn new(io: mio::unix::EventedFd, token: mio::Token) -> SignalEventSubscriber {
        SignalEventSubscriber {
            io: io,
            token: token,
        }
    }
}

impl<'a, S: CanSend<bool>> EventSubscriber<'a, S> for SignalEventSubscriber<'a> {
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
