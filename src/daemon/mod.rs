extern crate mio;
extern crate mio_uds;

use self::mio::{Poll, PollOpt, Ready, Token};
use self::mio_uds::{UnixListener, UnixStream};

use std::io::{Error, Read};
use std::net::Shutdown;

pub struct Daemon {
    listener : UnixListener
}

impl Daemon {
    pub fn new(poll : &Poll) -> Daemon
    {
        let listener = UnixListener::bind("/tmp/solanum").unwrap();
        poll.register(&listener, Token(0), Ready::readable(), PollOpt::edge()).expect("could not register listener with poll");

        Daemon { listener : listener }
    }

    pub fn handle_acceptor(&self)
    {
        let (mut stream, _) = self.listener.accept().unwrap().unwrap();
        let mut message = String::new();
        stream.read_to_string(&mut message).unwrap();
        stream.shutdown(Shutdown::Both);
    }
}
