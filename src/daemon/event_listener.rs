extern crate mio;
extern crate mio_uds;

use daemon::CommandProcessor;

use self::mio::{ Evented, Events, Poll, PollOpt, Ready, Token };

use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;

pub struct EventListener
{
    poll : Poll,
    events : Events,
    command_processor : CommandProcessor
}

impl EventListener
{
    pub fn new() -> io::Result<EventListener> {
        let poll = Poll::new().unwrap();
        let command_processor = try!(CommandProcessor::new(&poll));
        Ok(EventListener {
            poll: poll,
            events: Events::with_capacity(1024),
            command_processor: command_processor
        })
    }

    pub fn listen_for<E : ?Sized>(&self, io: &E, token : Token) -> io::Result<()> where E : Evented {
        self.poll.register(io, token, Ready::readable(), PollOpt::edge())
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
                    EventListener::clean_up();
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
                                EventListener::clean_up();
                                return Err(e);
                            }
                        }
                    },
                    Token(1) => {
                        info!("Signal received");
                        EventListener::clean_up();
                        return Ok(());
                    }
                    _ => {
                        error!("Unhandled token received");
                        EventListener::clean_up();
                        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Received event from unknown source"));
                    }
                }
            }
        }
    }

    fn clean_up()
    {
        fs::remove_file(Path::new("/tmp/solanum.pid")).unwrap();
    }
}
