use daemon::chrono::Duration;
use daemon::chrono::datetime::DateTime;
use daemon::chrono::offset::utc::UTC;

use daemon::clock::Clock;
use daemon::Command;
use daemon::PomodoroTransitioner;
use daemon::pomodoro::PomodoroStatus;
use daemon::pomodoros::Pomodoros;
use daemon::result::Error;
use daemon::result::Result;

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

    pub fn handle_command(&self, command: Command) -> Result<String> {
        match command {
            Command::Start(start_time, work_duration, break_duration, _) => {
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
                    -> Result<String> {
        let ref pomodoros = self.pomodoros;
        match pomodoros.most_recent() {
            Some(last_pomodoro) => {
                let now = self.clock.current_time();
                let mut updated_pomodoro = PomodoroTransitioner::transition(now, &last_pomodoro);
                if updated_pomodoro.status == PomodoroStatus::BreakPending {
                    updated_pomodoro = PomodoroTransitioner::transition(now, &updated_pomodoro);
                    updated_pomodoro = PomodoroTransitioner::transition(now, &updated_pomodoro);
                }
                try!(pomodoros.update(updated_pomodoro.id, updated_pomodoro));
            }
            None => {}
        }

        try!(pomodoros .create(start_time, work_duration, break_duration));
        pomodoros
            .most_recent()
            .ok_or(Error::from(String::from("Could not get the newly created pomodoro.")))
            .map(|pomodoro| format!(
                    "Pomodoro started at {}",
                    pomodoro.work_start_time.format("%F %H:%M:%S").to_string()
                    )
                )
    }

    fn handle_stop(&self) -> Result<String> {
        self.pomodoros
            .most_recent()
            .ok_or(Error::from(String::from("No pomodoro to stop.")))
            .map(|pomodoro| PomodoroTransitioner::transition(self.clock.current_time(), &pomodoro))
            .and_then(|pomodoro| self.pomodoros.update(pomodoro.id, pomodoro))
            .map(|_| String::from("Pomodoro aborted"))
    }

    fn handle_list(&self) -> Result<String> {
        self.pomodoros
            .last(5)
            .and_then(|pomodoros| {
                Ok(pomodoros.into_iter().fold(String::from(""), |acc, pomodoro| {
                    acc +
                        &format!("[{}]: {} ({})\n",
                        pomodoro.work_start_time.format("%F %H:%M:%S").to_string(),
                        pomodoro.status,
                        pomodoro.tags)
                }))
            })
    }

    fn handle_status(&self) -> Result<String> {
        let now = self.clock.current_time();
        self.pomodoros
            .most_recent()
            .ok_or(Error::from(String::from("No pomodoro to get the status of.")))
            .map(|pomodoro| {

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
            })
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
        fn create(&self, _: DateTime<UTC>, __: Duration, ___: Duration) -> Result<()> {
            Ok(())
        }

        fn last(&self, _: usize) -> Result<Vec<Pomodoro>> {
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

        fn update(&self, _: i32, __: Pomodoro) -> Result<()> {
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
                                     Duration::seconds(5),
                                     vec![]);
        let processor = CommandProcessor::new(clock_stub, pomodoros_stub);

        let result = processor.handle_command(command).unwrap();

        assert!(result == "Pomodoro started at 2000-01-01 00:00:00");
    }

    #[test]
    fn aborts_last_pomodoro_if_it_was_in_progress_and_not_yet_complete() {
        let mut scenario = mockers::Scenario::new();
        let pomodoros = scenario.create_mock_for::<Pomodoros>();
        let current_time = "2000-01-01T00:00:01+00:00".parse::<DateTime<UTC>>().unwrap();
        let clock_stub = ClockStub::new(current_time);
        let most_recent_pomodoro = create_pomodoro(
            "2000-01-01T00:00:00+00:00".parse::<DateTime<UTC>>().unwrap(),
            None,
            None,
            None,
            PomodoroStatus::InProgress,
        );
        let expected_update = create_pomodoro(
            "2000-01-01T00:00:00+00:00".parse::<DateTime<UTC>>().unwrap(),
            Some("2000-01-01T00:00:01+00:00".parse::<DateTime<UTC>>().unwrap()),
            None,
            None,
            PomodoroStatus::Aborted,
        );
        let command = Command::Start(current_time, Duration::seconds(5), Duration::seconds(5), vec![]);

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
        processor.handle_command(command).unwrap();
    }

    #[test]
    fn completes_last_pomodoro_if_it_was_in_progress_past_work_length_before_creating_a_new_one
        () {
        let mut scenario = mockers::Scenario::new();
        let pomodoros = scenario.create_mock_for::<Pomodoros>();
        let current_time = "2000-01-01T12:34:56+00:00".parse::<DateTime<UTC>>().unwrap();
        let clock_stub = ClockStub::new(current_time);
        let most_recent_pomodoro = create_pomodoro(
            "2000-01-01T00:00:00+00:00".parse::<DateTime<UTC>>().unwrap(),
            None,
            None,
            None,
            PomodoroStatus::InProgress
        );
        let expected_update = create_pomodoro(
            "2000-01-01T00:00:00+00:00".parse::<DateTime<UTC>>().unwrap(),
            Some("2000-01-01T00:00:05+00:00".parse::<DateTime<UTC>>().unwrap()),
            Some(current_time),
            Some(current_time),
            PomodoroStatus::Completed,
        );
        let command = Command::Start(current_time, Duration::seconds(5), Duration::seconds(5), vec![]);

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
        processor.handle_command(command).unwrap();
    }

    fn create_pomodoro(
        work_start_time: DateTime<UTC>,
        work_end_time: Option<DateTime<UTC>>,
        break_start_time: Option<DateTime<UTC>>,
        break_end_time: Option<DateTime<UTC>>,
        status: PomodoroStatus
        ) -> Pomodoro {
        Pomodoro {
            id: 0,
            work_start_time: work_start_time,
            work_end_time: work_end_time,
            break_start_time: break_start_time,
            break_end_time: break_end_time,
            work_length: Duration::seconds(5),
            break_length: Duration::seconds(5),
            tags: String::from(""),
            status: status
        }
    }
}
