extern crate chrono;

use self::chrono::DateTime;
use self::chrono::offset::utc::UTC;

pub trait Clock {
    fn current_time(&self) -> DateTime<UTC>;
}
