extern crate mio;
extern crate mio_uds;

use daemon::CanHandle;

use self::mio::{ Events, Poll, PollOpt, Ready };

use std::io;

pub struct EventPoller<'a> {
    poll: Poll,
    events: Events,
    subscriptions: Vec<&'a CanHandle>
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

    pub fn listen_for<'b>(&mut self, subscriber: &'b CanHandle) -> io::Result<()>
        where 'b : 'a
    {
        self.subscriptions.push(subscriber);
        self.poll.register(subscriber.io(), subscriber.token(), Ready::readable(), PollOpt::edge())
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
