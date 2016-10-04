/*extern crate mio;

use daemon::CanHandle;

use std::io;
use std::result;

pub struct EventSubscriber<'a, E : ?Sized, F : Fn(&'a E) -> result::Result<io::Result<()>, io::Result<()>>> where E : 'a + mio::Evented {
    pub io: &'a E,
    pub token: mio::Token,
    handler: F
}

impl<'a, E : ?Sized, F : Fn(&'a E) -> result::Result<io::Result<()>, io::Result<()>>> EventSubscriber<'a, E, F> where E : 'a + mio::Evented {
    pub fn new(io: &'a E, token: mio::Token, handler: F) -> EventSubscriber<'a, E, F> {
        EventSubscriber {
            io: io,
            token: token,
            handler: handler
        }
    }
}

impl<'a, E : ?Sized, F : Fn(&'a E) -> result::Result<io::Result<()>, io::Result<()>>> CanHandle for EventSubscriber<'a, E, F> where E : mio::Evented {
    fn handle(&self) -> result::Result<io::Result<()>, io::Result<()>> {
        (self.handler)(&self.io)
    }

    fn token(&self) -> mio::Token {
        self.token
    }

    fn io(&self) -> &mio::Evented {
        &self.io
    }
}
*/
