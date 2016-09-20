use std::io::{Error, Read};
use std::net::Shutdown;
use std::os::unix::net::{UnixListener, UnixStream};

pub struct Daemon {
}

impl Daemon {
    pub fn new() -> Daemon {
        Daemon { }
    }

    fn handle_incoming(mut stream : UnixStream) {
    }

    pub fn start(&self) {
    }
}
