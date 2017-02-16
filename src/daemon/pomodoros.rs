use daemon::pomodoro::Pomodoro;

use std::option::Option;

pub trait Pomodoros {
    fn most_recent(&self) -> Option<Pomodoro>;
}
