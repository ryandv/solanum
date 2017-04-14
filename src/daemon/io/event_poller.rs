use daemon::crossbeam;
use daemon::io::EventSubscriber;
use daemon::result::Error;
use daemon::result::Result;

use std::boxed::Box;
use std::collections::HashMap;
use std::io;
use std::ops::Deref;

use super::mio::{channel, Events, Poll, PollOpt, Ready, Token};

trait Joinable<T> {
    fn join(&self) -> T;
}

struct TrivialJoinable<T> {
    value: T
}

impl<T> TrivialJoinable<T> {
    pub fn new(value: T) -> TrivialJoinable<T> {
        TrivialJoinable { value: value }
    }
}

impl<T> Joinable<T> for TrivialJoinable<T> {
    fn join(&self) -> T {
        self.value
    }
}

impl<T> Joinable<T> for crossbeam::ScopedJoinHandle<T> {
    fn join(&self) -> T {
        self.join()
    }
}

pub struct EventPoller<'a> {
    poll: Poll,
    events: Events,
    subscriptions: HashMap<Token, &'a (EventSubscriber<'a, channel::Sender<bool>> + Sync)>,
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

    pub fn listen_for(&mut self, subscriber: &'a (EventSubscriber<'a, channel::Sender<bool>> + Sync)) -> io::Result<()>
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

            crossbeam::scope(|scope| {
                self.events.iter().map(|event| {
                    if event.token() == stop_token { return Box::new(TrivialJoinable::new(Err(Error::from(String::from("Received SIGINT"))))) as Box<Joinable<Result<()>>>; }

                    let stop_sender = stop_sender.clone();

                    match self.subscriptions.get(&event.token()) {
                        Some(subscriber) => Box::new(scope.spawn(move || subscriber.handle(stop_sender))) as Box<Joinable<Result<()>>>,
                        None => Box::new(TrivialJoinable::new(Err(Error::from(String::from("Received event from unknown source"))))) as Box<Joinable<Result<()>>>
                    }

                }).take_while(|result| (*result).join().is_ok())
            });
        }
    }
}
