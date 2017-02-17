extern crate chrono;

use daemon::pomodoro::Pomodoro;

use self::chrono::Duration;
use self::chrono::datetime::DateTime;
use self::chrono::offset::utc::UTC;

use std::vec::Vec;
use std::option::Option;

pub trait Pomodoros {
    fn create(&self, &DateTime<UTC>, Duration, Duration) -> Result<(), ()>;
    fn last(&self, usize) -> Result<Vec<Pomodoro>, ()>;
    fn most_recent(&self) -> Option<Pomodoro>;
    fn update(&self, i32, Pomodoro) -> Result<(), ()>;
}
