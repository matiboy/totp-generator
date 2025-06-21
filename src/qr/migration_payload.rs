use base32::{Alphabet, encode};

use crate::config::secrets::ConfigEntry;

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MigrationPayload {
    #[prost(message, repeated, tag = "1")]
    pub otp_parameters: ::prost::alloc::vec::Vec<OtpParameters>,
    #[prost(int32, tag = "2")]
    pub version: i32,
    #[prost(int32, tag = "3")]
    pub batch_size: i32,
    #[prost(int32, tag = "4")]
    pub batch_index: i32,
    #[prost(int32, tag = "5")]
    pub batch_id: i32,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OtpParameters {
    #[prost(bytes = "vec", tag = "1")]
    pub secret: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub issuer: ::prost::alloc::string::String,
    #[prost(int32, tag = "4")]
    pub algorithm: i32,
    #[prost(int32, tag = "5")]
    pub digits: i32,
    #[prost(int32, tag = "6")]
    pub r#type: i32,
    #[prost(int32, tag = "7")]
    pub counter: i32,
}

impl From<OtpParameters> for ConfigEntry {
    fn from(param: OtpParameters) -> Self {
        let secret = encode(Alphabet::RFC4648 { padding: false }, &param.secret);
        ConfigEntry::new(param.name, secret)
    }
}
