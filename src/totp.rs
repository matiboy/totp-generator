use base32::{decode, Alphabet};
use hmac::{Hmac, Mac};
use serde::Serialize;
use sha1::Sha1;
use std::{
    fmt,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Serialize, Debug)]
pub struct Totp {
    pub valid_until: u64,
    pub token: String,
    counter: u64,
}
impl Totp {
    pub fn new(secret: &str, time_step: u64, digits: Option<u8>) -> Totp {
        let mut totp = Totp {
            valid_until: 0,
            token: String::new(),
            counter: 0,
        };
        totp.refresh(secret, time_step, digits);
        totp
    }
    pub fn valid_duration(&self) -> u16 {
        (self.valid_until
            - SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()) as u16
    }

    pub fn needs_refresh(&self, time_step: u64) -> bool {
        let new_counter = get_counter(None, time_step);
        new_counter != self.counter
    }

    pub fn refresh(&mut self, secret: &str, time_step: u64, digits: Option<u8>) {
        let (otp, valid_until, counter) = generate_totp(secret, time_step, digits, None);
        self.token = otp;
        self.valid_until = valid_until;
        self.counter = counter;
    }
}

impl fmt::Display for Totp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} @ {}", self.token, self.valid_until)
    }
}

fn get_counter(timestamp: Option<u64>, time_step: u64) -> u64 {
    let current_time = timestamp.unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });
    
    current_time / time_step
}

fn generate_totp(
    secret: &str,
    time_step: u64,
    digits: Option<u8>,
    timestamp: Option<u64>,
) -> (String, u64, u64) {
    let digits = digits.unwrap_or(6);
    // Decode Base32 secret
    let secret_bytes =
        decode(Alphabet::RFC4648 { padding: false }, secret).expect("Invalid base32 secret");

    // Time counter (moving factor)
    let counter = get_counter(timestamp, time_step);
    let valid_until = (counter + 1) * time_step;

    // Convert counter to big-endian byte array
    let mut counter_bytes = [0u8; 8];
    for (i, byte) in counter.to_be_bytes().iter().enumerate() {
        counter_bytes[i] = *byte;
    }

    // HMAC-SHA1
    let mut mac = Hmac::<Sha1>::new_from_slice(&secret_bytes).unwrap();
    mac.update(&counter_bytes);
    let hmac_result = mac.finalize().into_bytes();

    // Dynamic Truncation
    let offset = (hmac_result[19] & 0x0f) as usize;
    let binary_code = ((hmac_result[offset] as u32 & 0x7f) << 24)
        | ((hmac_result[offset + 1] as u32) << 16)
        | ((hmac_result[offset + 2] as u32) << 8)
        | (hmac_result[offset + 3] as u32);

    // 6-digit code
    let otp = binary_code % 10u32.pow(digits as u32);
    (
        format!("{:0digits$}", otp, digits = digits as usize),
        valid_until,
        counter,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_totp_values() {
        let secret = "JBSWY3DPEHPK3PXP"; // This is not a real secret, just something random
        let digits = 6;
        let time_step = 30;

        let test_cases = vec![
            (1748742637, "879599", 1748742659, 58291421), // 1748742637 / 30 = 58291421
            (1748742663, "690726", 1748742679, 58291422),
            (1748742688, "690726", 1748742679, 58291422),
            (1748742714, "565959", 1748742739, 58291423),
            (1748742739, "295060", 1748742739, 58291424),
        ];

        for (timestamp, expected, expected_valid, expected_counter) in test_cases {
            let (otp, valid, counter) =
                generate_totp(secret, time_step, Some(digits), Some(timestamp));
            assert_eq!(otp, expected, "Failed for timestamp {}", timestamp);
            assert_eq!(
                valid, expected_valid,
                "Failed valid for timestamp {}",
                timestamp
            );
            assert_eq!(
                counter, expected_counter,
                "Failed counter for timestamp {}",
                timestamp
            );
        }
    }
}
