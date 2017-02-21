extern crate chrono;

use self::chrono::datetime::DateTime;
use self::chrono::offset::utc::UTC;

use daemon::clock::Clock;

pub struct SystemClock {
}

impl SystemClock {
    pub fn new() -> SystemClock {
        SystemClock { }
    }
}

impl Clock for SystemClock {
    fn current_time(&self) -> DateTime<UTC> {
        UTC::now()
    }
}

