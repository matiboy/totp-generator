use std::time::{Duration, SystemTime};

pub struct State {
    pub number_of_rows: u8,
    pub number_of_columns: u8,
    pub lock_password: Option< String >,
    pub unlocked_since: Option<SystemTime>,
    pub lock_after: Option<Duration>,
    pub secrets_path: String,
    pub buffer: String,
}

impl State  {
   pub fn default(secrets_path: String, lock_password: Option<String>, lock_after_seconds: u8) -> State {
       State {
        secrets_path,
        number_of_rows: 0,
        number_of_columns: 0,
        lock_password,
        unlocked_since: Some( SystemTime::now() ),
        lock_after: if lock_after_seconds > 0 { Some(Duration::from_secs(lock_after_seconds.into())) } else { None },
        buffer: "".to_owned(),
       }
   }
}
