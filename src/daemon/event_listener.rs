extern crate mio;
extern crate mio_uds;

use daemon::CommandProcessor;

use self::mio::{ Evented, Events, Poll, PollOpt, Ready, Token };

use std::error::Error;
use std::io;

pub struct EventListener<'a> {
    poll: Poll,
    events: Events,
    subscriptions: Vec<&'a CanHandle>,
    command_processor: CommandProcessor
}

pub struct EventSubscriptionDescriptor<'a, E : ?Sized, F : Fn(&'a E) -> io::Result<()>> where E : 'a + Evented {
    io: &'a E,
    token: Token,
    handler: F
}

pub trait CanHandle {
    fn handle (&self) -> io::Result<()>;
}

impl<'a, E : ?Sized, F : Fn(&'a E) -> io::Result<()>> EventSubscriptionDescriptor<'a, E, F> where E : 'a + Evented {
    pub fn new(io: &'a E, token: Token, handler: F) -> EventSubscriptionDescriptor<'a, E, F> {
        EventSubscriptionDescriptor {
            io: io,
            token: token,
            handler: handler
        }
    }
}

impl<'a, E : ?Sized, F : Fn(&'a E) -> io::Result<()>> CanHandle for EventSubscriptionDescriptor<'a, E, F> where E : Evented {
    fn handle(&self) -> io::Result<()> {
        (self.handler)(&self.io)
    }
}

impl<'a> EventListener<'a> {
    pub fn new() -> io::Result<EventListener<'a>> {
        let poll = Poll::new().unwrap();
        let command_processor = try!(CommandProcessor::new(&poll));
        Ok(EventListener {
            poll: poll,
            events: Events::with_capacity(1024),
            subscriptions: Vec::new(),
            command_processor: command_processor
        })
    }

    pub fn listen_for<E: ?Sized, F: Fn(&'a E) -> io::Result<()>>(&mut self, descriptor: &'a EventSubscriptionDescriptor<'a, E, F>) -> io::Result<()> where E : Evented, F : 'a {
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
                match event.token() {
                    Token(0) => {
                        match self.command_processor.handle_acceptor() {
                            Ok(_) => {
                            },
                            Err(e) => {
                                error!("{}", e.description());
                                return Err(e);
                            }
                        }
                    },
                    Token(1) => {
                        info!("Signal received");
                        return Ok(());
                    }
                    _ => {
                        error!("Unhandled token received");
                        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Received event from unknown source"));
                    }
                }
            }
        }
    }
}
