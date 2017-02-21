use daemon::pomodoro::Pomodoro;

use daemon::chrono::Duration;
use daemon::chrono::datetime::DateTime;
use daemon::chrono::offset::utc::UTC;

use std::vec::Vec;
use std::option::Option;

#[derive(Mock)]
pub trait Pomodoros {
    fn create(&self,
              start_time: DateTime<UTC>,
              start_duration: Duration,
              break_duration: Duration)
              -> Result<(), ()>;
    fn last(&self, count: usize) -> Result<Vec<Pomodoro>, ()>;
    fn most_recent(&self) -> Option<Pomodoro>;
    fn update(&self, id: i32, pomodoro: Pomodoro) -> Result<(), ()>;
}
