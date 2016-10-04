extern crate mio;
extern crate mio_uds;

use daemon::CommandProcessor;

use self::mio::{ Evented, Events, Poll, PollOpt, Ready, Token };

use std::error::Error;
use std::io;
use std::result;

pub struct EventPoller<'a> {
    poll: Poll,
    events: Events,
    subscriptions: Vec<&'a CanHandle>
}

pub struct EventSubscriptionDescriptor<'a, E : ?Sized, F : Fn(&'a E) -> result::Result<io::Result<()>, io::Result<()>>> where E : 'a + Evented {
    io: &'a E,
    token: Token,
    handler: F
}

pub trait CanHandle {
    fn handle(&self) -> result::Result<io::Result<()>, io::Result<()>>;
    fn token(&self) -> Token;
}

impl<'a, E : ?Sized, F : Fn(&'a E) -> result::Result<io::Result<()>, io::Result<()>>> EventSubscriptionDescriptor<'a, E, F> where E : 'a + Evented {
    pub fn new(io: &'a E, token: Token, handler: F) -> EventSubscriptionDescriptor<'a, E, F> {
        EventSubscriptionDescriptor {
            io: io,
            token: token,
            handler: handler
        }
    }
}

impl<'a, E : ?Sized, F : Fn(&'a E) -> result::Result<io::Result<()>, io::Result<()>>> CanHandle for EventSubscriptionDescriptor<'a, E, F> where E : Evented {
    fn handle(&self) -> result::Result<io::Result<()>, io::Result<()>> {
        (self.handler)(&self.io)
    }

    fn token(&self) -> Token {
        self.token
    }
}

impl<'a> EventPoller<'a> {
    pub fn new() -> io::Result<EventPoller<'a>> {
        let poll = Poll::new().unwrap();
        Ok(EventPoller {
            poll: poll,
            events: Events::with_capacity(1024),
            subscriptions: Vec::new()
        })
    }

    pub fn listen_for<'b, E: ?Sized, F: Fn(&'b E) -> result::Result<io::Result<()>, io::Result<()>>>(&mut self, descriptor: &'b EventSubscriptionDescriptor<'b, E, F>) -> io::Result<()>
        where 'b : 'a,
              E : Evented,
              F : 'b
    {
        self.subscriptions.push(descriptor);
        self.poll.register(descriptor.io, descriptor.token, Ready::readable(), PollOpt::edge())
    }

    /// Repeatedly poll for and handle incoming Events.
    /// Will return Ok if the dameon terminated gracefully after SIGTERM.
    /// Otherwise, will return Err with an Error indicating what happened.
    pub fn start_polling(&mut self) -> io::Result<()> {
        loop {
            match self.poll.poll(&mut self.events, None) {
                Ok(_) => {},
                Err(_) => {
                    error!("Could not poll for events");
                    return Err(io::Error::new(io::ErrorKind::InvalidInput, "Could not poll for events"));
                }
            }

            for event in self.events.iter() {
                let mut subscriptions_iter = self.subscriptions.iter();
                let handling_result = subscriptions_iter.
                    find(|subscription| subscription.token() == event.token()).
                    ok_or_else(|| {
                        error!("Unhandled token received");
                        Err(io::Error::new(io::ErrorKind::InvalidInput, "Received event from unknown source"))
                    }).
                    and_then(|subscription| subscription.handle());

                match handling_result {
                    Ok(_) => {},
                    Err(e) => { return e; }
                }
            }
        }
    }
}
