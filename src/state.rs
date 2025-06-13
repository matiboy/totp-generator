use std::{sync::Arc, time::{Duration, SystemTime}};

use tokio::sync::Mutex;

use crate::config::configuration::NumberStyle;

pub struct State {
    pub lock_password: Option< String >,
    pub unlocked_since: Option<SystemTime>,
    pub lock_after: Option<Duration>,
    pub secrets_path: Arc<Mutex<String>>,
    pub buffer: String,
    pub number_style: NumberStyle,
}

impl State  {
   pub fn default(secrets_path: Arc<Mutex<String>>, lock_password: Option<String>, lock_after_seconds: u16, number_style: NumberStyle) -> State {
       State {
        secrets_path,
        lock_password,
        unlocked_since: Some( SystemTime::now() ),
        lock_after: if lock_after_seconds > 0 { Some(Duration::from_secs(lock_after_seconds.into())) } else { None },
        buffer: "".to_owned(),
        number_style,
       }
   }
}
