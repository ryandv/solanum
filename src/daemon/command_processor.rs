extern crate chrono;

use self::chrono::datetime::DateTime;
use self::chrono::offset::utc::UTC;

use daemon::Command;
use daemon::PomodoroTransitioner;
use daemon::PomodoroQueryMapper;
use daemon::pomodoro::PomodoroStatus;

pub struct CommandProcessor {
}

impl CommandProcessor {
    pub fn new() -> CommandProcessor
    {
        CommandProcessor {
        }
    }

    pub fn handle_command(&self, command: Command) -> String
    {
        match command {
            Command::Start(start_time, work_duration, break_duration) => self.handle_start(&start_time, work_duration, break_duration),
            Command::Stop => self.handle_stop(),
            Command::List => self.handle_list(),
            Command::Status => self.handle_status()
        }
    }

    fn handle_start(&self, start_time : &DateTime<UTC>, work_duration: chrono::Duration, break_duration: chrono::Duration) -> String {
        let last_pomodoro = PomodoroQueryMapper::get_most_recent_pomodoro().
            map(|pomodoro| {
                let now = UTC::now();
                let mut updated_pomodoro = PomodoroTransitioner::transition(now, &pomodoro);
                if updated_pomodoro.status == PomodoroStatus::BreakPending {
                    updated_pomodoro = PomodoroTransitioner::transition(now, &updated_pomodoro);
                    updated_pomodoro = PomodoroTransitioner::transition(now, &updated_pomodoro);
                }
                updated_pomodoro
            }).
            and_then(|pomodoro| PomodoroQueryMapper::update_pomodoro(pomodoro.id, pomodoro) );

        let new_pomodoro = PomodoroQueryMapper::create_pomodoro(start_time, work_duration, break_duration).
            and_then(|_| PomodoroQueryMapper::get_most_recent_pomodoro());

        match new_pomodoro {
            Ok(pomodoro) => format!("Pomodoro started at {}", pomodoro.work_start_time.format("%F %H:%M:%S").to_string()),
            Err(_) => format!("Failed to start pomodoro.")
        }
    }

    fn handle_stop(&self) -> String {
        let result = PomodoroQueryMapper::get_most_recent_pomodoro().
            map(|pomodoro| PomodoroTransitioner::transition(UTC::now(), &pomodoro)).
            and_then(|pomodoro| PomodoroQueryMapper::update_pomodoro(pomodoro.id, pomodoro) );

        match result {
            Ok(_) => String::from("Pomodoro aborted"),
            Err(_) => String::from("Failed to abort pomodoro")
        }
    }

    fn handle_list(&self) -> String {
        let last_five_pomodoros = PomodoroQueryMapper::list_most_recent_pomodoros(5).
            and_then(|pomodoros| Ok(
                    pomodoros.into_iter().fold(String::from(""), |acc, pomodoro| {
                        acc + &format!("[{}]: {} ({})\n", pomodoro.work_start_time.format("%F %H:%M:%S").to_string(), pomodoro.status, pomodoro.tags)
                    })));

        match last_five_pomodoros {
            Ok(pomodoro_descriptions) => pomodoro_descriptions,
            Err(_) => String::from("Unable to list pomodoros")
        }
    }

    fn handle_status(&self) -> String {
        let now = UTC::now();
        let last_pomodoro = PomodoroQueryMapper::get_most_recent_pomodoro();

        match last_pomodoro {
            Ok(pomodoro) => {
                let work_time_remaining = (pomodoro.work_start_time + pomodoro.work_length) - now;
                let work_minutes_remaining = work_time_remaining.num_minutes();
                let work_seconds_remaining = work_time_remaining.num_seconds() - work_minutes_remaining * 60;
                let break_time_remaining = pomodoro.break_start_time.map(|start_time| (start_time + pomodoro.break_length) - now).unwrap_or(pomodoro.break_length);
                let break_minutes_remaining = break_time_remaining.num_minutes();
                let break_seconds_remaining = break_time_remaining.num_seconds() - break_minutes_remaining * 60;
                format!("{:02}:{:02} | {:02}:{:02}", work_minutes_remaining, work_seconds_remaining, break_minutes_remaining, break_seconds_remaining)
            }
            Err(_) => format!("Failed to start pomodoro.")
        }
    }
}
