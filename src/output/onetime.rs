use crate::config::secrets::get_secret;
use crate::totp::Totp;

pub fn one_time_mode(secrets_path: &str, arg: &str) {
    match get_secret(secrets_path, arg) {
        Some(entry) => {
            let otp = Totp::new(&entry.secret, entry.timestep, entry.digits);
            println!("Code for {}: {} (valid until {})", entry.name, otp, otp.valid_until);
        }
        None => println!("No matching entry found."),
    }
}
