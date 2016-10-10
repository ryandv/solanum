extern crate chrono;

use self::chrono::datetime::DateTime;
use self::chrono::offset::utc::UTC;

use daemon::Pomodoro;

pub struct PomodoroTransitioner {
}

impl PomodoroTransitioner {
    pub fn transition(current_time: DateTime<UTC>, pomodoro: Pomodoro) -> Pomodoro {
        if pomodoro.status == String::from("BREAKPENDING") {
            Pomodoro {
                id: pomodoro.id,
                work_start_time: pomodoro.work_start_time,
                work_end_time: pomodoro.work_end_time,
                break_start_time: Some(current_time),
                break_end_time: pomodoro.break_end_time,
                work_length: pomodoro.work_length,
                break_length: pomodoro.break_length,
                tags: pomodoro.tags,
                status: String::from("BREAK")
            }
        } else if pomodoro.status == String::from("BREAK") {
            Pomodoro {
                id: pomodoro.id,
                work_start_time: pomodoro.work_start_time,
                work_end_time: pomodoro.work_end_time,
                break_start_time: pomodoro.break_start_time,
                break_end_time: Some(current_time),
                work_length: pomodoro.work_length,
                break_length: pomodoro.break_length,
                tags: pomodoro.tags,
                status: String::from("COMPLETED")
            }
        } else {
            if current_time >= pomodoro.work_start_time + pomodoro.work_length {
                Pomodoro {
                    id: pomodoro.id,
                    work_start_time: pomodoro.work_start_time,
                    work_end_time: Some(pomodoro.work_start_time + pomodoro.work_length),
                    break_start_time: pomodoro.break_start_time,
                    break_end_time: pomodoro.break_end_time,
                    work_length: pomodoro.work_length,
                    break_length: pomodoro.break_length,
                    tags: pomodoro.tags,
                    status: String::from("BREAKPENDING")
                }
            } else {
                Pomodoro {
                    id: pomodoro.id,
                    work_start_time: pomodoro.work_start_time,
                    work_end_time: Some(current_time),
                    break_start_time: pomodoro.break_start_time,
                    break_end_time: pomodoro.break_end_time,
                    work_length: pomodoro.work_length,
                    break_length: pomodoro.break_length,
                    tags: pomodoro.tags,
                    status: String::from("ABORTED")
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::PomodoroTransitioner;

    use super::chrono::Duration;
    use super::chrono::datetime::DateTime;
    use super::chrono::offset::utc::UTC;

    use daemon::Pomodoro;

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
            status: String::from("INPROGRESS")
        };
        let transition_time = "2000-01-01T00:00:01+00:00".parse::<DateTime<UTC>>().unwrap();

        let updated_pomodoro = PomodoroTransitioner::transition(transition_time, pomodoro);

        assert!(updated_pomodoro.status == "ABORTED");
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
            status: String::from("INPROGRESS")
        };
        let transition_time = "2000-01-01T00:00:05+00:00".parse::<DateTime<UTC>>().unwrap();

        let updated_pomodoro = PomodoroTransitioner::transition(transition_time, pomodoro);

        assert!(updated_pomodoro.status == "BREAKPENDING");
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
            status: String::from("BREAKPENDING")
        };
        let transition_time = "2000-01-01T00:00:10+00:00".parse::<DateTime<UTC>>().unwrap();

        let updated_pomodoro = PomodoroTransitioner::transition(transition_time, pomodoro);

        assert!(updated_pomodoro.status == "BREAK");
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
            status: String::from("BREAK")
        };
        let transition_time = "2000-01-01T00:00:15+00:00".parse::<DateTime<UTC>>().unwrap();

        let updated_pomodoro = PomodoroTransitioner::transition(transition_time, pomodoro);

        assert!(updated_pomodoro.status == "COMPLETED");
        assert!(updated_pomodoro.break_end_time == Some(transition_time))
    }
}
