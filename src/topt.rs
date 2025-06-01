use std::{fmt, time::{SystemTime, UNIX_EPOCH}};
use hmac::{Hmac, Mac};
use sha1::Sha1;
use base32::{Alphabet, decode};

pub struct Totp {
    pub timestamp: u64,
    pub valid_until: u64,
    pub token: String,
}

impl fmt::Display for Totp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} @ {}", self.token, self.timestamp)
    }
}

pub fn generate_totp(secret: &str, time_step: u64, digits: Option<u8>, timestamp: Option<u64>) -> Totp {
    let digits = digits.unwrap_or(6);
    // Decode Base32 secret
    let secret_bytes = decode(Alphabet::RFC4648 { padding: false }, secret)
        .expect("Invalid base32 secret");

    // Time counter (moving factor)
    let current_time = timestamp.unwrap_or_else(|| SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
    let counter = current_time / time_step;
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
    Totp {
       token: format!("{:0digits$}", otp, digits = digits as usize),
       timestamp: current_time,
       valid_until,
    }
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
            (1748742637, "879599"),
            (1748742663, "690726"),
            (1748742688, "690726"),
            (1748742714, "565959"),
            (1748742739, "295060"),
        ];

        for (timestamp, expected) in test_cases {
            let otp = generate_totp(secret, time_step, Some(digits), Some(timestamp));
            assert_eq!(otp.token, expected, "Failed for timestamp {}", otp);
        }
    }
}
