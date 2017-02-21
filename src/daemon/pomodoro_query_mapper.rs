extern crate postgres;

use daemon::chrono::Duration;
use daemon::chrono::datetime::DateTime;
use daemon::chrono::offset::utc::UTC;

use daemon::pomodoro::Pomodoro;
use daemon::pomodoros::Pomodoros;
use daemon::pomodoro::PomodoroStatus;

use self::postgres::rows;

use std::iter::FromIterator;
use std::option::Option;
use std::result::Result;
use std::error::Error;

pub struct PomodoroQueryMapper {}

impl PomodoroQueryMapper {
    pub fn new() -> PomodoroQueryMapper {
        PomodoroQueryMapper {}
    }

    pub fn create_pomodoro(&self,
                           start_time: DateTime<UTC>,
                           work_duration: Duration,
                           break_duration: Duration)
                           -> Result<(), ()> {
        let conn = postgres::Connection::connect("postgres://postgres@localhost:5432/solanum_test",
                                                 postgres::SslMode::None)
            .unwrap();
        let work_length = work_duration.num_seconds();
        let break_length = break_duration.num_seconds();
        let result = conn.execute("INSERT INTO pomodoros(
                work_start_time,
                \
                                   work_end_time,
                break_start_time,
                \
                                   break_end_time,
                work_length,
                \
                                   break_length,
                tags,
                status
            \
                                   ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                                  &[&start_time,
                                    &None as &Option<DateTime<UTC>>,
                                    &None as &Option<DateTime<UTC>>,
                                    &None as &Option<DateTime<UTC>>,
                                    &work_length as &i64,
                                    &break_length as &i64,
                                    &String::from(""),
                                    &PomodoroStatus::InProgress.to_string()]);

        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("{}", e.description());
                Err(())
            }
        }
    }

    pub fn get_most_recent_pomodoro(&self) -> Result<Pomodoro, ()> {
        match self.list_most_recent_pomodoros(1) {
            Ok(mut pomodoros) => {
                if pomodoros.is_empty() {
                    Err(())
                } else {
                    Ok(pomodoros.remove(0))
                }
            }
            Err(_) => Err(()),
        }
    }

    pub fn list_most_recent_pomodoros(&self, limit: usize) -> Result<Vec<Pomodoro>, ()> {
        let conn = postgres::Connection::connect("postgres://postgres@localhost:5432/solanum_test",
                                                 postgres::SslMode::None)
            .unwrap();

        let most_recent_results: rows::Rows = try!((&conn)
            .query("SELECT id, work_start_time, work_end_time, break_start_time, \
                    break_end_time, work_length, break_length, status, tags FROM pomodoros \
                    ORDER BY work_start_time DESC",
                   &[])
            .or_else(|err| {
                error!("{}", err.description());
                Err(())
            }));

        Ok(Vec::from_iter(most_recent_results.iter().take(limit).map(|pomodoro| {
            let id: i32 = pomodoro.get(0);
            let work_start_time: DateTime<UTC> = pomodoro.get(1);
            let work_end_time: Option<DateTime<UTC>> = pomodoro.get(2);
            let break_start_time: Option<DateTime<UTC>> = pomodoro.get(3);
            let break_end_time: Option<DateTime<UTC>> = pomodoro.get(4);
            let work_length: i64 = pomodoro.get(5);
            let break_length: i64 = pomodoro.get(6);
            let status: String = pomodoro.get(7);
            let tags: String = pomodoro.get(8);

            Pomodoro {
                id: id,
                work_start_time: work_start_time,
                work_end_time: work_end_time,
                break_start_time: break_start_time,
                break_end_time: break_end_time,
                work_length: Duration::seconds(work_length),
                break_length: Duration::seconds(break_length),
                status: PomodoroStatus::from(status),
                tags: tags,
            }
        })))
    }

    pub fn update_pomodoro(&self, id: i32, pomodoro: Pomodoro) -> Result<(), ()> {
        let conn = postgres::Connection::connect("postgres://postgres@localhost:5432/solanum_test",
                                                 postgres::SslMode::None)
            .unwrap();
        (&conn)
            .execute("UPDATE pomodoros SET
                    work_start_time = $2,
                    \
                      work_end_time = $3,
                    break_start_time = $4,
                    \
                      break_end_time = $5,
                    work_length = $6,
                    \
                      break_length = $7,
                    tags = $8,
                    \
                      status = $9
                WHERE id = $1",
                     &[&id,
                       &pomodoro.work_start_time,
                       &pomodoro.work_end_time,
                       &pomodoro.break_start_time,
                       &pomodoro.break_end_time,
                       &pomodoro.work_length.num_seconds() as &i64,
                       &pomodoro.break_length.num_seconds() as &i64,
                       &String::from(""),
                       &pomodoro.status.to_string()])
            .or_else(|err| {
                error!("{}", err.description());
                Err(())
            })
            .map(|_| ())
    }
}

impl Pomodoros for PomodoroQueryMapper {
    fn create(&self,
              start_time: DateTime<UTC>,
              work_duration: Duration,
              break_duration: Duration)
              -> Result<(), ()> {
        self.create_pomodoro(start_time, work_duration, break_duration)
    }

    fn last(&self, count: usize) -> Result<Vec<Pomodoro>, ()> {
        self.list_most_recent_pomodoros(count)
    }

    fn most_recent(&self) -> Option<Pomodoro> {
        self.get_most_recent_pomodoro().ok()
    }

    fn update(&self, id: i32, pomodoro: Pomodoro) -> Result<(), ()> {
        self.update_pomodoro(id, pomodoro)
    }
}
