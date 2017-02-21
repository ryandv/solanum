use daemon::chrono::Duration;
use daemon::chrono::datetime::DateTime;
use daemon::chrono::offset::utc::UTC;

use daemon::clock::Clock;
use daemon::Command;
use daemon::PomodoroTransitioner;
use daemon::pomodoro::PomodoroStatus;
use daemon::pomodoros::Pomodoros;

pub struct CommandProcessor<C: Clock, P: Pomodoros> {
    clock: C,
    pomodoros: P,
}

impl<C: Clock, P: Pomodoros> CommandProcessor<C, P> {
    pub fn new(clock: C, pomodoros: P) -> CommandProcessor<C, P> {
        CommandProcessor {
            clock: clock,
            pomodoros: pomodoros,
        }
    }

    pub fn handle_command(&self, command: Command) -> String {
        match command {
            Command::Start(start_time, work_duration, break_duration) => {
                self.handle_start(start_time, work_duration, break_duration)
            }
            Command::Stop => self.handle_stop(),
            Command::List => self.handle_list(),
            Command::Status => self.handle_status(),
        }
    }

    fn handle_start(&self,
                    start_time: DateTime<UTC>,
                    work_duration: Duration,
                    break_duration: Duration)
                    -> String {
        let ref pomodoros = self.pomodoros;
        let last_pomodoro = pomodoros.most_recent()
            .ok_or(())
            .map(|pomodoro| {
                let now = self.clock.current_time();
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
            .map(|pomodoro| PomodoroTransitioner::transition(self.clock.current_time(), &pomodoro))
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
        let now = self.clock.current_time();
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
    extern crate mockers;

    use super::*;

    use daemon::Command;
    use daemon::clock::Clock;
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
        fn create(&self, _: DateTime<UTC>, __: Duration, ___: Duration) -> Result<(), ()> {
            Ok(())
        }

        fn last(&self, _: usize) -> Result<Vec<Pomodoro>, ()> {
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

        fn update(&self, _: i32, __: Pomodoro) -> Result<(), ()> {
            Ok(())
        }
    }

    struct ClockStub {
        fake_time: DateTime<UTC>,
    }

    impl ClockStub {
        fn new(fake_time: DateTime<UTC>) -> ClockStub {
            ClockStub { fake_time: fake_time }
        }
    }

    impl Clock for ClockStub {
        fn current_time(&self) -> DateTime<UTC> {
            self.fake_time
        }
    }

    #[test]
    fn creates_a_new_pomodoro() {
        let pomodoros_stub = PomodorosStub::new();
        let clock_stub =
            ClockStub::new("2000-01-01T00:00:00+00:00".parse::<DateTime<UTC>>().unwrap());
        let command = Command::Start(clock_stub.current_time(),
                                     Duration::seconds(5),
                                     Duration::seconds(5));
        let processor = CommandProcessor::new(clock_stub, pomodoros_stub);

        let result = processor.handle_command(command);

        assert!(result == "Pomodoro started at 2000-01-01 00:00:00");
    }

    #[test]
    fn completes_last_pomodoro_if_it_was_in_progress_past_work_length_before_creating_a_new_one
        () {
        let mut scenario = mockers::Scenario::new();
        let pomodoros = scenario.create_mock_for::<Pomodoros>();
        let current_time = "2017-01-01T12:34:56+00:00".parse::<DateTime<UTC>>().unwrap();
        let clock_stub = ClockStub::new(current_time);
        let most_recent_pomodoro = Pomodoro {
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
        let expected_update = Pomodoro {
            id: 0,
            work_start_time: "2000-01-01T00:00:00+00:00".parse::<DateTime<UTC>>().unwrap(),
            work_end_time: Some("2000-01-01T00:00:05+00:00".parse::<DateTime<UTC>>().unwrap()),
            break_start_time: Some(current_time),
            break_end_time: Some(current_time),
            work_length: Duration::seconds(5),
            break_length: Duration::seconds(5),
            tags: String::from(""),
            status: PomodoroStatus::Completed,
        };
        let command = Command::Start(current_time, Duration::seconds(5), Duration::seconds(5));

        scenario.expect(pomodoros.most_recent_call()
            .and_return_clone(Some(most_recent_pomodoro))
            .times(2));
        scenario.expect(pomodoros.update_call(expected_update.id, expected_update)
            .and_return(Ok(())));
        scenario.expect(pomodoros.create_call(
            current_time,
            Duration::seconds(5),
            Duration::seconds(5)
            ).
            and_return(Ok(()))
        );

        let processor = CommandProcessor::new(clock_stub, pomodoros);
        processor.handle_command(command);
    }
}
