extern crate time;

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
