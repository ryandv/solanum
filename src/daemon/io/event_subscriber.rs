use daemon::io::mio;

use daemon::result::Error;
use daemon::result::Result;

use std::convert::From;

pub trait CanSend<T> {
    fn send(&self, t: T) -> Result<()>;
}

impl CanSend<bool> for mio::channel::Sender<bool> where Error: From<mio::channel::SendError<bool>> {
    fn send(&self, t: bool) -> Result<()> {
        self.send(t).map_err(|e| Error::from(e))
    }
}

pub trait EventSubscriber<'a, S: CanSend<bool>> {
    fn handle(&self, s: S) -> Result<()>;
    fn token(&self) -> mio::Token;
    fn io(&self) -> &mio::Evented;
}
