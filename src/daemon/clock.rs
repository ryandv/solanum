use daemon::chrono::DateTime;
use daemon::chrono::offset::utc::UTC;

pub trait Clock {
    fn current_time(&self) -> DateTime<UTC>;
}
