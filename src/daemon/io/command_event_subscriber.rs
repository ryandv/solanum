extern crate mio;
extern crate mio_uds;

use daemon::chrono::offset::utc::UTC;

use daemon::clock::Clock;
use daemon::Command;
use daemon::CommandProcessor;
use daemon::io::{CanHandle, CanSend};
use daemon::pomodoros::Pomodoros;
use daemon::result::Error;
use daemon::result::Result;

use self::mio_uds::UnixListener;
use self::mio_uds::UnixStream;

use std::fs;
use std::io::{Error as IOError, ErrorKind, Read, Write};
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
               -> Result<CommandEventSubscriber<C, P>> {
        match UnixListener::bind("/tmp/solanum") {
            Ok(listener) => {
                Ok(CommandEventSubscriber {
                    io: listener,
                    command_processor: command_processor,
                    token: token,
                })
            }
            Err(e) => Err(Error::from(e))
        }
    }

    fn process_stream(&self, stream: &mut UnixStream) -> Result<()> {
        let mut buf: [u8; 1024] = [0; 1024];
        try!(stream.read(&mut buf));
        let codepoints = Vec::from_iter(buf.to_vec()
            .into_iter()
            .take_while(|codepoint| *codepoint != (0 as u8)));
        let message = try!(String::from_utf8(codepoints)
            .map_err(|_| IOError::new(ErrorKind::InvalidInput, "failed to parse UTF-8 command")));
        let command = try!(Command::from_string(UTC::now(), message)
            .map_err(|_| IOError::new(ErrorKind::InvalidInput, "failed to parse command string")));

        try!(self.command_processor
            .handle_command(command)
            .and_then(|response| {
                let result = stream
                    .write_all(response.as_bytes())
                    .and_then(|_| { stream.shutdown(Shutdown::Both) })
                    .map_err(|e| Error::from(e));

                info!("Handled command");
                result
            }));

        Ok(())
    }
}

impl<'a, C: Clock, P: Pomodoros, S: CanSend<bool>> CanHandle<'a, S> for CommandEventSubscriber<C, P> {
    fn handle(&self, _: S) -> Result<()> {
        self
            .io
            .accept()
            .map_err(|e| Error::from(e))
            .and_then(|acceptor| {
                match acceptor {
                    Some((mut stream, _)) => {
                        self.process_stream(&mut stream)
                            .map_err(|e| Error::from(e))
                    }
                    None => {
                        warn!("Expected connection but got none");
                        Ok(())
                    }
                }
            })
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
