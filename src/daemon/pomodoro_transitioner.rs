extern crate chrono;

use self::chrono::datetime::DateTime;
use self::chrono::offset::utc::UTC;

use daemon::Pomodoro;

pub struct PomodoroTransitioner {
}

impl PomodoroTransitioner {
    pub fn transition(current_time: DateTime<UTC>, pomodoro: Pomodoro) -> Pomodoro {
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
                status: String::from("COMPLETED")
            }
        } else {
            Pomodoro {
                id: pomodoro.id,
                work_start_time: pomodoro.work_start_time,
                work_end_time: pomodoro.work_end_time,
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

        assert!(updated_pomodoro.status == "ABORTED")
    }

    #[test]
    fn completes_a_pomodoro_transitioned_after_work_duration_has_elapsed() {
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

        assert!(updated_pomodoro.status == "COMPLETED");
        assert!(updated_pomodoro.work_end_time == Some(transition_time))
    }
}
