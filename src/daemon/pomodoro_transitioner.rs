use daemon::chrono::datetime::DateTime;
use daemon::chrono::offset::utc::UTC;

use daemon::pomodoro::Pomodoro;
use daemon::pomodoro::PomodoroStatus;

pub struct PomodoroTransitioner {}

impl PomodoroTransitioner {
    pub fn transition(current_time: DateTime<UTC>, pomodoro: &Pomodoro) -> Pomodoro {
        let new_pomodoro = pomodoro.clone();

        match (*pomodoro).status {
            PomodoroStatus::Aborted => new_pomodoro,
            PomodoroStatus::Completed => new_pomodoro,
            PomodoroStatus::BreakPending => {
                PomodoroTransitioner::start_break(current_time, new_pomodoro)
            }
            PomodoroStatus::Break => {
                PomodoroTransitioner::complete_pomodoro(current_time, new_pomodoro)
            }
            PomodoroStatus::InProgress => {
                if current_time >= pomodoro.work_start_time + pomodoro.work_length {
                    PomodoroTransitioner::finish_working(current_time, new_pomodoro)
                } else {
                    PomodoroTransitioner::abort_pomodoro(current_time, new_pomodoro)
                }
            }
        }
    }

    fn start_break(current_time: DateTime<UTC>, pomodoro: Pomodoro) -> Pomodoro {
        let mut pomodoro = pomodoro;
        pomodoro.break_start_time = Some(current_time);
        pomodoro.status = PomodoroStatus::Break;
        pomodoro
    }

    fn complete_pomodoro(current_time: DateTime<UTC>, pomodoro: Pomodoro) -> Pomodoro {
        let mut pomodoro = pomodoro;
        pomodoro.break_end_time = Some(current_time);
        pomodoro.status = PomodoroStatus::Completed;
        pomodoro
    }

    fn finish_working(current_time: DateTime<UTC>, pomodoro: Pomodoro) -> Pomodoro {
        let mut pomodoro = pomodoro;
        pomodoro.work_end_time = Some(pomodoro.work_start_time + pomodoro.work_length);
        pomodoro.status = PomodoroStatus::BreakPending;
        pomodoro
    }

    fn abort_pomodoro(current_time: DateTime<UTC>, pomodoro: Pomodoro) -> Pomodoro {
        let mut pomodoro = pomodoro;
        pomodoro.work_end_time = Some(current_time);
        pomodoro.status = PomodoroStatus::Aborted;
        pomodoro
    }
}

#[cfg(test)]
mod test {
    use daemon::chrono::Duration;

    use super::*;

    #[test]
    fn aborts_a_pomodoro_transitioned_before_work_duration_has_elapsed() {
        let pomodoro = Pomodoro {
            id: 0,
            work_start_time: "2000-01-01T00:00:00+00:00".parse::<DateTime<UTC>>().unwrap(),
            work_end_time: None,
            break_start_time: None,
            break_end_time: None,
            work_length: Duration::seconds(5),
            break_length: Duration::seconds(5),
            tags: String::from(""),
            status: PomodoroStatus::InProgress,
        };
        let transition_time = "2000-01-01T00:00:01+00:00".parse::<DateTime<UTC>>().unwrap();

        let updated_pomodoro = PomodoroTransitioner::transition(transition_time, &pomodoro);

        assert!(updated_pomodoro.status == PomodoroStatus::Aborted);
        assert!(updated_pomodoro.work_end_time == Some(transition_time));
    }

    #[test]
    fn moves_a_pomodoro_transitioned_after_work_duration_has_elapsed_to_break_pending() {
        let pomodoro = Pomodoro {
            id: 0,
            work_start_time: "2000-01-01T00:00:00+00:00".parse::<DateTime<UTC>>().unwrap(),
            work_end_time: None,
            break_start_time: None,
            break_end_time: None,
            work_length: Duration::seconds(5),
            break_length: Duration::seconds(5),
            tags: String::from(""),
            status: PomodoroStatus::InProgress,
        };
        let transition_time = "2000-01-01T00:00:05+00:00".parse::<DateTime<UTC>>().unwrap();

        let updated_pomodoro = PomodoroTransitioner::transition(transition_time, &pomodoro);

        assert!(updated_pomodoro.status == PomodoroStatus::BreakPending);
        assert!(updated_pomodoro.work_end_time == Some(transition_time))
    }

    #[test]
    fn starts_a_break_for_pomodoros_transitioned_from_breakpending() {
        let pomodoro = Pomodoro {
            id: 0,
            work_start_time: "2000-01-01T00:00:00+00:00".parse::<DateTime<UTC>>().unwrap(),
            work_end_time: Some("2000-01-01T00:00:05+00:00".parse::<DateTime<UTC>>().unwrap()),
            break_start_time: None,
            break_end_time: None,
            work_length: Duration::seconds(5),
            break_length: Duration::seconds(5),
            tags: String::from(""),
            status: PomodoroStatus::BreakPending,
        };
        let transition_time = "2000-01-01T00:00:10+00:00".parse::<DateTime<UTC>>().unwrap();

        let updated_pomodoro = PomodoroTransitioner::transition(transition_time, &pomodoro);

        assert!(updated_pomodoro.status == PomodoroStatus::Break);
        assert!(updated_pomodoro.break_start_time == Some(transition_time))
    }

    #[test]
    fn completes_pomodoros_transitioned_after_break_time_has_elapsed() {
        let pomodoro = Pomodoro {
            id: 0,
            work_start_time: "2000-01-01T00:00:00+00:00".parse::<DateTime<UTC>>().unwrap(),
            work_end_time: Some("2000-01-01T00:00:05+00:00".parse::<DateTime<UTC>>().unwrap()),
            break_start_time: Some("2000-01-01T00:00:10+00:00".parse::<DateTime<UTC>>().unwrap()),
            break_end_time: None,
            work_length: Duration::seconds(5),
            break_length: Duration::seconds(5),
            tags: String::from(""),
            status: PomodoroStatus::Break,
        };
        let transition_time = "2000-01-01T00:00:15+00:00".parse::<DateTime<UTC>>().unwrap();

        let updated_pomodoro = PomodoroTransitioner::transition(transition_time, &pomodoro);

        assert!(updated_pomodoro.status == PomodoroStatus::Completed);
        assert!(updated_pomodoro.break_end_time == Some(transition_time))
    }

    #[test]
    fn does_nothing_to_an_aborted_pomodoro() {
        let pomodoro = Pomodoro {
            id: 0,
            work_start_time: "2000-01-01T00:00:00+00:00".parse::<DateTime<UTC>>().unwrap(),
            work_end_time: Some("2000-01-01T00:00:01+00:00".parse::<DateTime<UTC>>().unwrap()),
            break_start_time: None,
            break_end_time: None,
            work_length: Duration::seconds(5),
            break_length: Duration::seconds(5),
            tags: String::from(""),
            status: PomodoroStatus::Aborted,
        };
        let transition_time = "2000-01-01T00:00:10+00:00".parse::<DateTime<UTC>>().unwrap();

        let updated_pomodoro = PomodoroTransitioner::transition(transition_time, &pomodoro);

        assert!(updated_pomodoro == pomodoro);
    }

    #[test]
    fn does_nothing_to_a_completed_pomodoro() {
        let pomodoro = Pomodoro {
            id: 0,
            work_start_time: "2000-01-01T00:00:00+00:00".parse::<DateTime<UTC>>().unwrap(),
            work_end_time: Some("2000-01-01T00:00:05+00:00".parse::<DateTime<UTC>>().unwrap()),
            break_start_time: Some("2000-01-01T00:00:10+00:00".parse::<DateTime<UTC>>().unwrap()),
            break_end_time: Some("2000-01-01T00:00:15+00:00".parse::<DateTime<UTC>>().unwrap()),
            work_length: Duration::seconds(5),
            break_length: Duration::seconds(5),
            tags: String::from(""),
            status: PomodoroStatus::Completed,
        };
        let transition_time = "2000-01-01T00:00:30+00:00".parse::<DateTime<UTC>>().unwrap();

        let updated_pomodoro = PomodoroTransitioner::transition(transition_time, &pomodoro);

        println!("{:?} {:?}", updated_pomodoro, pomodoro);
        assert!(updated_pomodoro == pomodoro);
    }
}
