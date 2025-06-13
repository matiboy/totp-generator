use crate::config::secrets::ConfigFile;
use crate::totp::Totp;

pub async fn one_time_mode(secrets_path: &str, arg: &str) -> anyhow::Result<String> {
    if arg.is_empty() {
        tracing::warn!("No argument provided for one-time mode; this is meant to be the code/index of the secret. This could lead to unexpected behavior.");
    }
    let mut cf = ConfigFile::new(secrets_path.to_owned());
    let (_, secrets) = cf.load().await?;
    let totp = ConfigFile::get_secret(&secrets, arg)
        .map(|entry| Totp::new(&entry.secret, entry.timestep, entry.digits))?;
    Ok(format!("{}\nValid until: {}", totp.token, totp.valid_until))
}
