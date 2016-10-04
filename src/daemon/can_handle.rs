extern crate mio;

use std::io;
use std::result;

pub trait CanHandle {
    fn handle(&self) -> result::Result<io::Result<()>, io::Result<()>>;
    fn token(&self) -> mio::Token;
    fn io(&self) -> &mio::Evented;
}
