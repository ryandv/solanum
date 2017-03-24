use daemon;

use daemon::io::mio::channel;

use std::fmt;
use std::fmt::Debug;
use std::io;
use std::result;
use std::string::{FromUtf8Error, String};

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    DbConnectError(daemon::postgres::error::ConnectError),
    DbError(daemon::postgres::error::Error),
    FailedStopError(channel::SendError<bool>),
    GenericError(String),
    IoError(io::Error),
    CommandFromUtf8Error(FromUtf8Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        match *self {
            Error::DbConnectError(ref e) => write!(f, "Database connection error: {}", e),
            Error::DbError(ref e) => write!(f, "Database error: {}", e),
            Error::FailedStopError(_) => write!(f, "Failed to stop polling for events."),
            Error::GenericError(ref e) => write!(f, "{}", e),
            Error::IoError(ref e) => write!(f, "IO error: {}", e),
            Error::CommandFromUtf8Error(ref e) => write!(f, "Could not parse command from UTF-8: {}", e)
        }
    }
}

impl From<daemon::postgres::error::ConnectError> for Error {
    fn from(err: daemon::postgres::error::ConnectError) -> Error {
        Error::DbConnectError(err)
    }
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
        Error::GenericError(err)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Error {
        Error::CommandFromUtf8Error(err)
    }
}
