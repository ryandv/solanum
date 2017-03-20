use daemon;

use daemon::io::mio::channel;

use daemon::command;

use std::io;
use std::result;
use std::string::{FromUtf8Error, String};

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    DbError(daemon::postgres::error::Error),
    FailedStopError(channel::SendError<bool>),
    InternalError(String),
    IoError(io::Error),
    InvalidCommandError(command::InvalidCommandString),
    CommandFromUtf8Error(FromUtf8Error),
}

impl From<daemon::postgres::error::Error> for Error {
    fn from(err: daemon::postgres::error::Error) -> Error {
        Error::DbError(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

impl From<channel::SendError<bool>> for Error {
    fn from(err: channel::SendError<bool>) -> Error {
        Error::FailedStopError(err)
    }
}

impl From<String> for Error {
    fn from(err: String) -> Error {
        Error::InternalError(err)
    }
}

impl From<command::InvalidCommandString> for Error {
    fn from(err: command::InvalidCommandString) -> Error {
        Error::InvalidCommandError(err)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Error {
        Error::CommandFromUtf8Error(err)
    }
}
