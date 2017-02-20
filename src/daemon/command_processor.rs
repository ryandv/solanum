extern crate chrono;

use self::chrono::datetime::DateTime;
use self::chrono::offset::utc::UTC;

use std::ops::Deref;

use daemon::Command;
use daemon::PomodoroTransitioner;
use daemon::PomodoroQueryMapper;
use daemon::pomodoro::PomodoroStatus;
use daemon::pomodoros::Pomodoros;

pub struct CommandProcessor<P: Pomodoros> {
    pomodoros: P,
}

impl<P: Pomodoros> CommandProcessor<P> {
    pub fn new(pomodoros: P) -> CommandProcessor<P> {
        CommandProcessor { pomodoros: pomodoros }
    }

    pub fn handle_command(&self, command: Command) -> String {
        match command {
            Command::Start(start_time, work_duration, break_duration) => {
                self.handle_start(&start_time, work_duration, break_duration)
            }
            Command::Stop => self.handle_stop(),
            Command::List => self.handle_list(),
            Command::Status => self.handle_status(),
        }
    }

    fn handle_start(&self,
                    start_time: &DateTime<UTC>,
                    work_duration: chrono::Duration,
                    break_duration: chrono::Duration)
                    -> String {
        let ref pomodoros = self.pomodoros;
        let last_pomodoro = pomodoros.most_recent()
            .ok_or(())
            .map(|pomodoro| {
                let now = UTC::now();
                let mut updated_pomodoro = PomodoroTransitioner::transition(now, &pomodoro);
                if updated_pomodoro.status == PomodoroStatus::BreakPending {
                    updated_pomodoro = PomodoroTransitioner::transition(now, &updated_pomodoro);
                    updated_pomodoro = PomodoroTransitioner::transition(now, &updated_pomodoro);
                }
                updated_pomodoro
            })
            .and_then(|pomodoro| pomodoros.update(pomodoro.id, pomodoro));

        let new_pomodoro = pomodoros.create(start_time, work_duration, break_duration)
            .and_then(|_| pomodoros.most_recent().ok_or(()));

        match new_pomodoro {
            Ok(pomodoro) => {
                format!("Pomodoro started at {}",
                        pomodoro.work_start_time.format("%F %H:%M:%S").to_string())
            }
            Err(_) => format!("Failed to start pomodoro."),
        }
    }

    fn handle_stop(&self) -> String {
        let result = self.pomodoros
            .most_recent()
            .ok_or(())
            .map(|pomodoro| PomodoroTransitioner::transition(UTC::now(), &pomodoro))
            .and_then(|pomodoro| self.pomodoros.update(pomodoro.id, pomodoro));

        match result {
            Ok(_) => String::from("Pomodoro aborted"),
            Err(_) => String::from("Failed to abort pomodoro"),
        }
    }

    fn handle_list(&self) -> String {
        let last_five_pomodoros = self.pomodoros.last(5).and_then(|pomodoros| {
            Ok(pomodoros.into_iter().fold(String::from(""), |acc, pomodoro| {
                acc +
                &format!("[{}]: {} ({})\n",
                         pomodoro.work_start_time.format("%F %H:%M:%S").to_string(),
                         pomodoro.status,
                         pomodoro.tags)
            }))
        });

        match last_five_pomodoros {
            Ok(pomodoro_descriptions) => pomodoro_descriptions,
            Err(_) => String::from("Unable to list pomodoros"),
        }
    }

    fn handle_status(&self) -> String {
        let now = UTC::now();
        let last_pomodoro = self.pomodoros.most_recent().ok_or(());

        match last_pomodoro {
            Ok(pomodoro) => {
                let work_time_remaining = (pomodoro.work_start_time + pomodoro.work_length) - now;
                let work_minutes_remaining = work_time_remaining.num_minutes();
                let work_seconds_remaining = work_time_remaining.num_seconds() -
                                             work_minutes_remaining * 60;
                let break_time_remaining = pomodoro.break_start_time
                    .map(|start_time| (start_time + pomodoro.break_length) - now)
                    .unwrap_or(pomodoro.break_length);
                let break_minutes_remaining = break_time_remaining.num_minutes();
                let break_seconds_remaining = break_time_remaining.num_seconds() -
                                              break_minutes_remaining * 60;
                format!("{:02}:{:02} | {:02}:{:02}",
                        work_minutes_remaining,
                        work_seconds_remaining,
                        break_minutes_remaining,
                        break_seconds_remaining)
            }
            Err(_) => format!("Failed to start pomodoro."),
        }
    }
}

#[cfg(test)]
mod test {
    use super::CommandProcessor;

    use super::chrono::Duration;
    use super::chrono::datetime::DateTime;
    use super::chrono::offset::utc::UTC;

    use daemon::Command;
    use daemon::pomodoro::Pomodoro;
    use daemon::pomodoro::PomodoroStatus;
    use daemon::pomodoros::Pomodoros;

    struct PomodorosStub {}

    impl PomodorosStub {
        fn new() -> PomodorosStub {
            PomodorosStub {}
        }
    }

    impl Pomodoros for PomodorosStub {
        fn create(&self,
                  start_time: &DateTime<UTC>,
                  work_duration: Duration,
                  break_duration: Duration)
                  -> Result<(), ()> {
            Ok(())
        }

        fn last(&self, count: usize) -> Result<Vec<Pomodoro>, ()> {
            Ok(Vec::new())
        }

        fn most_recent(&self) -> Option<Pomodoro> {
            Some(Pomodoro {
                id: 0,
                work_start_time: "2000-01-01T00:00:00+00:00".parse::<DateTime<UTC>>().unwrap(),
                work_end_time: None,
                break_start_time: None,
                break_end_time: None,
                work_length: Duration::seconds(5),
                break_length: Duration::seconds(5),
                tags: String::from(""),
                status: PomodoroStatus::InProgress,
            })
        }

        fn update(&self, id: i32, pomodoro: Pomodoro) -> Result<(), ()> {
            Ok(())
        }
    }

    #[test]
    fn creates_a_new_pomodoro() {
        let pomodoros_stub = PomodorosStub::new();
        let processor = CommandProcessor::new(pomodoros_stub);
        let command = Command::Start("2000-01-01T00:00:00+00:00".parse::<DateTime<UTC>>().unwrap(),
                                     Duration::seconds(5),
                                     Duration::seconds(5));

        let result = processor.handle_command(command);

        assert!(result == "Pomodoro started at 2000-01-01 00:00:00");
    }
}
