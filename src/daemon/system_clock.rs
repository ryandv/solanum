use daemon::chrono::datetime::DateTime;
use daemon::chrono::offset::utc::UTC;

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

