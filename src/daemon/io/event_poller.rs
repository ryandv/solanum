use daemon::io::EventSubscriber;
use daemon::result::Error;
use daemon::result::Result;

use std::collections::HashMap;
use std::io;

use super::mio::{channel, Events, Poll, PollOpt, Ready, Token};

pub struct EventPoller<'a> {
    poll: Poll,
    events: Events,
    subscriptions: HashMap<Token, &'a EventSubscriber<'a, channel::Sender<bool>>>,
}

impl<'a> EventPoller<'a> {
    pub fn new() -> io::Result<EventPoller<'a>> {
        let poll = try!(Poll::new());
        Ok(EventPoller {
            poll: poll,
            events: Events::with_capacity(1024),
            subscriptions: HashMap::new(),
        })
    }

    pub fn listen_for(&mut self, subscriber: &'a EventSubscriber<'a, channel::Sender<bool>>) -> io::Result<()>
    {
        self.subscriptions.insert(subscriber.token(), subscriber);
        self.poll.register(subscriber.io(),
                           subscriber.token(),
                           Ready::readable(),
                           PollOpt::edge())
    }

    /// Repeatedly poll for and handle incoming Events.
    /// Will return Ok if the dameon terminated gracefully after SIGTERM.
    /// Otherwise, will return Err with an Error indicating what happened.
    pub fn start_polling(&mut self) -> Result<()> {
        let (stop_sender, stop_receiver) = channel::channel::<bool>();
        let stop_token = Token(2);

        try!(self.poll.register(&stop_receiver, stop_token, Ready::readable(), PollOpt::edge()));

        loop {
            try!(self.poll.poll(&mut self.events, None)
                .map_err(|e| Error::from(e)));

            for event in self.events.iter() {
                if event.token() == stop_token { return Ok(()); }

                let stop_sender = stop_sender.clone();

                try!(self.subscriptions
                    .get(&event.token())
                    .ok_or(Error::from(String::from("Received event from unknown source")))
                    .and_then(|subscriber| subscriber.handle(stop_sender)));
            }
        }
    }
}
