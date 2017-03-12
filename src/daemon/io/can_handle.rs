use super::mio;

use daemon::result::Result;

use std::io;
use std::result;

pub trait CanHandle {
    fn handle(&self) -> result::Result<Result<()>, io::Result<()>>;
    fn token(&self) -> mio::Token;
    fn io(&self) -> &mio::Evented;
}
