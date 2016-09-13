use std::io::{Error, Read};
use std::net::Shutdown;
use std::os::unix::net::{UnixListener, UnixStream};

pub struct Daemon {
    listener : UnixListener
}

impl Daemon {
    pub fn new() -> Daemon {
        Daemon { listener: UnixListener::bind("/tmp/solanum").unwrap() }
    }

    fn handle_incoming(mut stream : UnixStream) {
        let mut message = String::new();
        stream.read_to_string(&mut message).unwrap();
        stream.shutdown(Shutdown::Both);
    }

    pub fn start(&self) {
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => { Daemon::handle_incoming(stream); },
                Err(_) => {
                    break;
                }
            }
        };
    }
}
