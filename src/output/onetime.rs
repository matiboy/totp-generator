use crate::config::secrets::get_config;
use crate::topt::generate_totp;

pub fn one_time_mode(config_path: &str, arg: &str) {
    match get_config(config_path, arg) {
        Some(entry) => {
            let otp = generate_totp(&entry.secret, 6, None, None);
            println!("Code for {}: {} (valid until {})", entry.name, otp, otp.valid_until);
        }
        None => println!("No matching entry found."),
    }
}
