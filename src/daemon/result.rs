use daemon;

use daemon::command;

use std::io;
use std::error;
use std::result;
use std::str;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    DbError(daemon::postgres::error::Error),
    IoError(io::Error),
    MalformedCommandError(CommandError)
}

#[derive(Debug)]
pub enum CommandError {
    InvalidCommandError(command::InvalidCommandString),
    Utf8Error(str::Utf8Error),
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
