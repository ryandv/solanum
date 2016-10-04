extern crate mio;
extern crate mio_uds;

use mio::{ Events, Poll, PollOpt, Ready, Token };
use mio::unix::EventedFd;

pub struct EventListener
{
    poll : Poll,
    events : Events
}

impl EventListener
{
    pub fn new() -> EventListener
    {
        EventListener {
            poll: Poll::new().unwrap,
            events: Events::with_capacity(1024)
        }
    }
}
