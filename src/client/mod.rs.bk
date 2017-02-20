use std::io::{ Error, Read, Write };
use std::time::Duration;
use std::os::unix::net::UnixStream;

pub struct Client {
}

impl Client {
    pub fn new() -> Client {
        Client {}
    }
    pub fn send_message(&self, message : String) -> Result<String, Error> {
        let mut stream = try!(UnixStream::connect("/tmp/solanum"));
        try!(stream.set_write_timeout(Some(Duration::new(5, 0))));
        try!(stream.set_read_timeout(Some(Duration::new(5, 0))));
        try!(stream.write_all(message.as_bytes()));
        let mut response = String::new();
        try!(stream.read_to_string(&mut response));
        Ok(response)
    }
}
