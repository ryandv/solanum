use daemon::crossbeam;
use daemon::io::EventSubscriber;
use daemon::result::Error;
use daemon::result::Result;

use std::boxed::Box;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io;
use std::mem;
use std::ops::Deref;

use super::mio::{channel, Events, Poll, PollOpt, Ready, Token};

trait Deferred<T> {
    fn resolve(self: Box<Self>) -> Option<T>;
}

struct ImmediateDeferred<T> {
    value: Option<T>
}

impl<T> ImmediateDeferred<T> {
    pub fn new(value: T) -> ImmediateDeferred<T> {
        ImmediateDeferred { value: Some(value) }
    }
}

impl<T> Deferred<T> for ImmediateDeferred<T> {
    fn resolve(self: Box<Self>) -> Option<T> {
        self.value
    }
}

impl<T> Deferred<T> for crossbeam::ScopedJoinHandle<T> {
    fn resolve(self: Box<Self>) -> Option<T> {
        Some(self.join())
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

        let mut boxed_deferreds: Vec<Box<Deferred<Result<()>>>> = vec![];

        crossbeam::scope(|scope| {
            'outer: loop {
                try!(self.poll.poll(&mut self.events, None)
                     .map_err(|e| Error::from(e)));


                for event in self.events.iter() {
                    if event.token() == stop_token {
                        info!("Received SIGINT");
                        boxed_deferreds.push(Box::new(ImmediateDeferred::new(Err(Error::from(String::from("Received SIGINT"))))));
                        break 'outer;
                    }

                    let stop_sender = stop_sender.clone();

                    match self.subscriptions.get(&event.token()) {
                        Some(subscriber) => { boxed_deferreds.push(Box::new(scope.spawn(move || subscriber.handle(stop_sender)))); },
                        None => {
                            info!("Received event from unknown source");
                            boxed_deferreds.push(Box::new(ImmediateDeferred::new(Err(Error::from(String::from("Received event from unknown source"))))));
                            break 'outer;
                        }
                    };
                }
            };

            info!("Exiting event loop soon");

            let mut iter = boxed_deferreds.into_iter();
            let results: Result<Vec<()>> = iter.filter_map(|deferred| {
                match deferred.resolve().unwrap() {
                    Ok(_) => None,
                    Err(e) => Some(Err(e))
                }
            }).take(1).collect();
            return results.map(|_| ());
        })
    }
}
