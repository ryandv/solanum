extern crate chrono;

use self::chrono::datetime::DateTime;
use self::chrono::offset::utc::UTC;

use std::fmt;

pub struct Pomodoro {
    pub id: i32,
    pub work_start_time: chrono::datetime::DateTime<UTC>,
    pub work_end_time: Option<chrono::datetime::DateTime<UTC>>,
    pub break_start_time: Option<chrono::datetime::DateTime<UTC>>,
    pub break_end_time: Option<chrono::datetime::DateTime<UTC>>,
    pub work_length: chrono::Duration,
    pub break_length: chrono::Duration,
    pub tags: String,
    pub status: PomodoroStatus
}

#[derive(PartialEq, Eq)]
pub enum PomodoroStatus {
    IN_PROGRESS,
    ABORTED,
    BREAK_PENDING,
    BREAK,
    COMPLETED
}

impl fmt::Display for PomodoroStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PomodoroStatus::IN_PROGRESS => write!(f, "IN_PROGRESS"),
            PomodoroStatus::ABORTED => write!(f, "ABORTED"),
            PomodoroStatus::BREAK_PENDING => write!(f, "BREAK_PENDING"),
            PomodoroStatus::BREAK => write!(f, "BREAK"),
            PomodoroStatus::COMPLETED => write!(f, "COMPLETED"),
        }
    }
}

impl From<String> for PomodoroStatus {
    fn from(string: String) -> PomodoroStatus {
        match string.as_str() {
            "IN_PROGRESS" => PomodoroStatus::IN_PROGRESS,
            "BREAK_PENDING" => PomodoroStatus::BREAK_PENDING,
            "BREAK" => PomodoroStatus::BREAK,
            "COMPLETED" => PomodoroStatus::COMPLETED,
            _ => PomodoroStatus::ABORTED
        }
    }
}
