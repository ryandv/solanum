extern crate chrono;
extern crate mio;
extern crate mio_uds;

use daemon::clock::Clock;
use daemon::Command;
use daemon::CommandProcessor;
use daemon::io::CanHandle;
use daemon::pomodoros::Pomodoros;

use self::mio_uds::UnixListener;
use self::mio_uds::UnixStream;

use std::fs;
use std::io;
use std::result;
use std::io::{Error, ErrorKind, Read, Write};
use std::iter::FromIterator;
use std::net::Shutdown;
use std::path::Path;

pub struct CommandEventSubscriber<C: Clock, P: Pomodoros> {
    io: UnixListener,
    command_processor: CommandProcessor<C, P>,
    token: mio::Token,
}

impl<C: Clock, P: Pomodoros> CommandEventSubscriber<C, P> {
    pub fn new(command_processor: CommandProcessor<C, P>,
               token: mio::Token)
               -> io::Result<CommandEventSubscriber<C, P>> {
        let listener = try!(UnixListener::bind("/tmp/solanum"));
        Ok(CommandEventSubscriber {
            io: listener,
            command_processor: command_processor,
            token: token,
        })
    }

    fn process_stream(&self, stream: &mut UnixStream) -> io::Result<()> {
        let mut buf: [u8; 1024] = [0; 1024];
        try!(stream.read(&mut buf));
        let codepoints = Vec::from_iter(buf.to_vec()
            .into_iter()
            .take_while(|codepoint| *codepoint != (0 as u8)));
        let message = try!(String::from_utf8(codepoints)
            .map_err(|_| Error::new(ErrorKind::InvalidInput, "failed to parse UTF-8 command")));
        let command = try!(Command::from_string(chrono::offset::utc::UTC::now(), message)
            .map_err(|_| Error::new(ErrorKind::InvalidInput, "failed to parse command string")));

        let response = self.command_processor.handle_command(command);

        try!(stream.write_all(response.as_bytes()));
        try!(stream.shutdown(Shutdown::Both));
        info!("Handled command");
        Ok(())
    }
}

impl<C: Clock, P: Pomodoros> CanHandle for CommandEventSubscriber<C, P> {
    fn handle(&self) -> result::Result<io::Result<()>, io::Result<()>> {
        match self.io.accept() {
            Ok(acceptor) => {
                match acceptor {
                    Some((mut stream, _)) => Ok(self.process_stream(&mut stream)),
                    None => {
                        warn!("Expected connection but got none");
                        Ok(Ok(()))
                    }
                }
            }
            Err(e) => Err(Err(e)),
        }
    }

    fn token(&self) -> mio::Token {
        self.token
    }

    fn io(&self) -> &mio::Evented {
        &self.io
    }
}

impl<C: Clock, P: Pomodoros> Drop for CommandEventSubscriber<C, P> {
    fn drop(&mut self) {
        // TODO: log errors instead of just silently discarding.
        // right now, silently discarding errors to ensure listener is recursively dropped.
        match fs::remove_file(Path::new("/tmp/solanum")) {
            Ok(_) => {}
            Err(_) => {}
        }
    }
}
