extern crate postgres;
extern crate time;

use self::postgres::rows;

use std::result;
use std::error::Error;

pub struct PomodoroQueryMapper {
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
            map_err(|_| ())
        );

        let most_recent_start_timespec: time::Timespec  = most_recent_result.get(0).get(0);

        Ok(time::at(most_recent_start_timespec))
    }
}
