use crate::config::secrets::get_secret;
use crate::totp::Totp;

pub fn one_time_mode(secrets_path: &str, arg: &str) -> anyhow::Result<String> {
    if arg.is_empty() {
        tracing::warn!("No argument provided for one-time mode; this is meant to be the code/index of the secret. This could lead to unexpected behavior.");
    }
    let totp = get_secret(secrets_path, arg)
        .map(|entry| Totp::new(&entry.secret, entry.timestep, entry.digits))?;
    Ok(format!("{}\nValid until: {}", totp.token, totp.valid_until))
}
