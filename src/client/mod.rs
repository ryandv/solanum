extern crate unix_socket;

use std::io::{Error, Write};
use std::time::Duration;
use self::unix_socket::UnixStream;

pub struct Client {
}

impl Client {
    pub fn send_message(&self) -> Result<(), Error> {
        let mut stream = try!(UnixStream::connect("/tmp/solanum"));
        try!(stream.set_write_timeout(Some(Duration::new(5, 0))));
        try!(stream.set_read_timeout(Some(Duration::new(5, 0))));
        stream.write_all("Hello".as_bytes())
    }
}
