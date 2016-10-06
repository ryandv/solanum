extern crate mio;
extern crate mio_uds;

use daemon::{ CanHandle, Command, CommandProcessor };

use self::mio_uds::UnixListener;

use std::io;
use std::result;
use std::io::{ Read, Write };
use std::iter::FromIterator;
use std::net::Shutdown;

pub struct CommandEventSubscriber {
    io: UnixListener,
    command_processor: CommandProcessor,
    token: mio::Token
}

impl CommandEventSubscriber {
    pub fn new(io: UnixListener, command_processor: CommandProcessor, token: mio::Token) -> CommandEventSubscriber {
        CommandEventSubscriber {
            io: io,
            command_processor: command_processor,
            token: token
        }
    }
}

impl CanHandle for CommandEventSubscriber {
    fn handle(&self) -> result::Result<io::Result<()>, io::Result<()>> {
        match self.io.accept() {
            Ok(acceptor) => {
                match acceptor {
                    Some((mut stream, _)) => {
                        let mut buf : [u8; 1024] = [0; 1024];
                        stream.read(&mut buf).unwrap();
                        let codepoints = Vec::from_iter(buf.to_vec().into_iter().take_while(|codepoint| *codepoint != (0 as u8)));
                        let message = String::from_utf8(codepoints).unwrap();
                        let command = Command::from_string(message).unwrap();

                        let response = self.command_processor.handle_acceptor(command);

                        stream.write_all(response.as_bytes());
                        stream.shutdown(Shutdown::Both).unwrap();
                        info!("Handled command");

                        Ok(Ok(()))
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
