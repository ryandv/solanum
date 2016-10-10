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
    InProgress,
    Aborted,
    BreakPending,
    Break,
    Completed
}

impl fmt::Display for PomodoroStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PomodoroStatus::InProgress => write!(f, "InProgress"),
            PomodoroStatus::Aborted => write!(f, "Aborted"),
            PomodoroStatus::BreakPending => write!(f, "BreakPending"),
            PomodoroStatus::Break => write!(f, "Break"),
            PomodoroStatus::Completed => write!(f, "Completed"),
        }
    }
}

impl From<String> for PomodoroStatus {
    fn from(string: String) -> PomodoroStatus {
        match string.as_str() {
            "InProgress" => PomodoroStatus::InProgress,
            "BreakPending" => PomodoroStatus::BreakPending,
            "Break" => PomodoroStatus::Break,
            "Completed" => PomodoroStatus::Completed,
            _ => PomodoroStatus::Aborted
        }
    }
}
