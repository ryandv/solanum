extern crate chrono;

use self::chrono::datetime::DateTime;
use self::chrono::offset::utc::UTC;

pub struct Pomodoro {
    pub id: i32,
    pub work_start_time: chrono::datetime::DateTime<UTC>,
    pub work_end_time: Option<chrono::datetime::DateTime<UTC>>,
    pub break_start_time: Option<chrono::datetime::DateTime<UTC>>,
    pub break_end_time: Option<chrono::datetime::DateTime<UTC>>,
    pub work_length: chrono::Duration,
    pub break_length: chrono::Duration,
    pub tags: String,
    pub status: String
}
