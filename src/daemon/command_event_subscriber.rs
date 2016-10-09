extern crate mio;
extern crate mio_uds;

use daemon::{ CanHandle, Command, CommandProcessor };

use self::mio_uds::UnixListener;
use self::mio_uds::UnixStream;

use std::fs;
use std::io;
use std::result;
use std::io::{ Error, ErrorKind, Read, Write };
use std::iter::FromIterator;
use std::net::Shutdown;
use std::path::Path;

pub struct CommandEventSubscriber {
    io: UnixListener,
    command_processor: CommandProcessor,
    token: mio::Token
}

impl CommandEventSubscriber {
    pub fn new(command_processor: CommandProcessor, token: mio::Token) -> io::Result<CommandEventSubscriber> {
        let listener = try!(UnixListener::bind("/tmp/solanum"));
        Ok(CommandEventSubscriber {
            io: listener,
            command_processor: command_processor,
            token: token
        })
    }

    fn process_stream(&self, stream: &mut UnixStream) -> io::Result<()> {
        let mut buf : [u8; 1024] = [0; 1024];
        try!(stream.read(&mut buf));
        let codepoints = Vec::from_iter(buf.to_vec().into_iter().take_while(|codepoint| *codepoint != (0 as u8)));
        let message = try!(String::from_utf8(codepoints).map_err(|_| Error::new(ErrorKind::InvalidInput, "failed to parse UTF-8 command")));
        let command = try!(Command::from_string(message));

        let response = self.command_processor.handle_command(command);

        stream.write_all(response.as_bytes());
        try!(stream.shutdown(Shutdown::Both));
        info!("Handled command");
        Ok(())
    }
}

impl CanHandle for CommandEventSubscriber {
    fn handle(&self) -> result::Result<io::Result<()>, io::Result<()>> {
        match self.io.accept() {
            Ok(acceptor) => {
                match acceptor {
                    Some((mut stream, _)) => {
                        Ok(self.process_stream(&mut stream))
                    }
                    None => {
                        warn!("Expected connection but got none");
                        Ok(Ok(()))
                    }
                }
            },
            Err(e) => {
                Err(Err(e))
            }
        }
    }

    fn token(&self) -> mio::Token {
        self.token
    }

    fn io(&self) -> &mio::Evented {
        &self.io
    }
}

impl Drop for CommandEventSubscriber {
    fn drop(&mut self)
    {
        // TODO: log errors instead of just silently discarding.
        // right now, silently discarding errors to ensure listener is recursively dropped.
        match fs::remove_file(Path::new("/tmp/solanum")) {
            Ok(_) => {},
            Err(_) => {}
        }
    }
}
