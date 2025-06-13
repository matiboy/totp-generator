use std::{sync::Arc, time::{Duration, SystemTime}};

use actix_web::cookie::time::format_description::well_known::iso8601::Config;
use tokio::sync::{Mutex, RwLock};

use crate::config::{configuration::NumberStyle, secrets::ConfigFile};

pub struct State {
    pub lock_password: Option< String >,
    pub unlocked_since: Option<SystemTime>,
    pub lock_after: Option<Duration>,
    pub secrets_cf: Arc<RwLock<ConfigFile>>,
    pub buffer: String,
    pub number_style: NumberStyle,
}

impl State  {
   pub fn default(secrets_cf: Arc<RwLock<ConfigFile>>, lock_password: Option<String>, lock_after_seconds: u16, number_style: NumberStyle) -> State {
       State {
        secrets_cf,
        lock_password,
        unlocked_since: Some( SystemTime::now() ),
        lock_after: if lock_after_seconds > 0 { Some(Duration::from_secs(lock_after_seconds.into())) } else { None },
        buffer: "".to_owned(),
        number_style,
       }
   }
}
