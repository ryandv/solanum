use daemon;

use std::error;
use std::result;

pub type Result<T> = result::Result<T, Error>;

pub enum Error {
    DbError(daemon::postgres::error::Error)
}

impl From<daemon::postgres::error::Error> for Error {
    fn from(err: daemon::postgres::error::Error) -> Error {
        Error::DbError(err)
    }
}
