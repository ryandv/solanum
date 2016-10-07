extern crate postgres;
extern crate time;

use self::postgres::rows;

use std::iter::FromIterator;
use std::result;
use std::error::Error;

pub struct PomodoroQueryMapper {
}

pub struct Pomodoro {
    pub id: i32,
    pub work_start_time: time::Tm,
    pub work_end_time: Option<time::Tm>,
    pub break_start_time: Option<time::Tm>,
    pub break_end_time: Option<time::Tm>,
    pub work_length: time::Duration,
    pub break_length: time::Duration,
    pub tags: String,
    pub status: String
}

impl PomodoroQueryMapper {
    pub fn create_pomodoro(start_time : &time::Tm, work_duration: time::Duration, break_duration: time::Duration) -> result::Result<(), ()> {
        let conn = postgres::Connection::connect("postgres://postgres@localhost:5432/solanum_test", postgres::SslMode::None).unwrap();
        let work_length = work_duration.num_seconds();
        let break_length = break_duration.num_seconds();
        let result = conn.execute(
            "INSERT INTO pomodoros(
                work_start_time,
                work_end_time,
                break_start_time,
                break_end_time,
                work_length,
                break_length,
                tags,
                status
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            &[&start_time.to_timespec(), &None as &Option<time::Timespec>, &None as &Option<time::Timespec>, &None as &Option<time::Timespec>, &work_length as &i64, &break_length as &i64, &String::from(""), &String::from("STARTED")]
            );

        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("{}", e.description());
                Err(())
            }
        }
    }

    pub fn get_most_recent_pomodoro_start_time() -> result::Result<time::Tm, ()> {
        let conn = postgres::Connection::connect("postgres://postgres@localhost:5432/solanum_test", postgres::SslMode::None).unwrap();

        let most_recent_result: rows::Rows = try!(
            (&conn).
            query("SELECT work_start_time FROM pomodoros ORDER BY work_start_time DESC LIMIT 1", &[]).
            or_else(|err| { error!("{}", err.description()); Err(()) })
        );

        let most_recent_start_timespec: time::Timespec  = most_recent_result.get(0).get(0);

        Ok(time::at(most_recent_start_timespec))
    }

    pub fn get_most_recent_pomodoro() -> result::Result<Pomodoro, ()> {
        match PomodoroQueryMapper::list_most_recent_pomodoros(1) {
            Ok(mut pomodoros) => if pomodoros.is_empty() {
                Err(())
            } else {
                Ok(pomodoros.remove(0))
            },
            Err(_) => Err(())
        }
    }

    pub fn list_most_recent_pomodoros(limit: usize) -> result::Result<Vec<Pomodoro>, ()> {
        let conn = postgres::Connection::connect("postgres://postgres@localhost:5432/solanum_test", postgres::SslMode::None).unwrap();

        let most_recent_results: rows::Rows = try!(
            (&conn).
            query("SELECT id, work_start_time, work_end_time, break_start_time, break_end_time, work_length, break_length, status, tags FROM pomodoros ORDER BY work_start_time DESC", &[]).
            or_else(|err| { error!("{}", err.description()); Err(())})
        );

        Ok(Vec::from_iter(most_recent_results.iter().take(limit).map(|pomodoro| {
            let id: i32 = pomodoro.get(0);
            let work_start_time: time::Timespec = pomodoro.get(1);
            let work_end_time: Option<time::Timespec> = pomodoro.get(2);
            let break_start_time: Option<time::Timespec> = pomodoro.get(3);
            let break_end_time: Option<time::Timespec> = pomodoro.get(4);
            let work_length: i64 = pomodoro.get(5);
            let break_length: i64 = pomodoro.get(6);
            let status: String = pomodoro.get(7);
            let tags: String = pomodoro.get(8);

            Pomodoro {
                id: id,
                work_start_time: time::at(work_start_time),
                work_end_time: work_end_time.map(|timespec| time::at(timespec)),
                break_start_time: break_start_time.map(|timespec| time::at(timespec)),
                break_end_time: break_end_time.map(|timespec| time::at(timespec)),
                work_length: time::Duration::seconds(work_length),
                break_length: time::Duration::seconds(break_length),
                status: status,
                tags: tags
            }
        })))
    }

    pub fn stop_pomodoro(id: i32) -> result::Result<(), ()> {
        let conn = postgres::Connection::connect("postgres://postgres@localhost:5432/solanum_test", postgres::SslMode::None).unwrap();
        (&conn).
            execute(
                "UPDATE pomodoros SET status = $2
                    WHERE id = $1",
                    &[&id, &String::from("COMPLETED")]
            ).
            or_else(|err| { error!("{}", err.description()); Err(()) }).
            map(|_| ())
    }
}