use std::io::{Error, Read};
use std::net::Shutdown;
use std::os::unix::net::{UnixListener, UnixStream};

pub struct Daemon {
}

impl Daemon {
    fn handle_incoming(mut stream : UnixStream) {
        let mut message = String::new();
        stream.read_to_string(&mut message).unwrap();
        stream.shutdown(Shutdown::Both);
    }

    pub fn start(&self) {
        let listener = UnixListener::bind("/tmp/solanum").unwrap();
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => { Daemon::handle_incoming(stream); },
                Err(_) => {
                    break;
                }
            }
        };
    }
}
