use std::path::PathBuf;

use anyhow::{Context, Result};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;

use image::{DynamicImage, GrayImage};
#[cfg(feature = "configure")]
use prost::Message;
#[cfg(feature = "configure")]
use rqrr::PreparedImage;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use url::Url;

use super::migration_payload::{MigrationPayload, OtpParameters};

pub struct QrDecoder;
// Example URI:
// otpauth-migration://offline?data=CjMKCkhlbGxvId6tvu8SGFRlc3QxOnRlc3QxQGV4YW1wbGUxLmNvbRoFVGVzdDEgASgBMAIKMwoKSGVsbG8h3q2%2B8BIYVGVzdDI6dGVzdDJAZXhhbXBsZTIuY29tGgVUZXN0MiABKAEwAgozCgpIZWxsbyHerb7xEhhUZXN0Mzp0ZXN0M0BleGFtcGxlMy5jb20aBVRlc3QzIAEoATACEAEYASAAKI3orYEE
#[cfg(feature = "configure")]
impl QrDecoder {
    pub fn parse_google_auth_export(uri: &str) -> Result<Vec<OtpParameters>> {
        let url = Url::parse(uri).context("Invalid QR code URI")?;

        let data = url
            .query_pairs()
            .find(|(k, _)| k == "data")
            .map(|(_, v)| v.into_owned())
            .context("Missing `data` query parameter")?;
        let decoded = STANDARD.decode(data).context("Base64 decode failed")?;
        let payload = MigrationPayload::decode(decoded.as_slice())
            .context("Failed to parse protobuf payload")?;

        let entries = payload.otp_parameters.into_iter().collect();
        Ok(entries)
    }
    pub async fn decode_from_file(path: PathBuf) -> Result<String> {
        // Async read file into memory
        let p = path.clone();
        let mut file = File::open(path)
            .await
            .with_context(|| format!("Failed to open file: {p:?}"))?;

        let mut buffer = vec![];
        file.read_to_end(&mut buffer)
            .await
            .context("Failed to read image file")?;

        // Decode image
        let img = image::load_from_memory(&buffer).context("Failed to decode image from memory")?;

        Self::decode_image(&img)
    }

    fn decode_image(img: &DynamicImage) -> Result<String> {
        let gray: GrayImage = img.to_luma8();

        let mut img = PreparedImage::prepare(gray);
        let grids = img.detect_grids();

        if grids.is_empty() {
            anyhow::bail!("No QR codes found");
        }

        let (_, content) = grids[0].decode().context("Failed to decode QR content")?;

        Ok(content)
    }
}
