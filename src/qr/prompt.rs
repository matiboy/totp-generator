use crate::{config::{configuration::Origin, secrets::ConfigEntry}, qr::reader::QrDecoder};
use std::{io::{self, Write as _}, path::PathBuf};


pub async fn generate_configuration(
    from_image: PathBuf,
    prompt: bool,
    origin: Origin,
    use_zbar: bool,
) -> anyhow::Result<()> {
    let content = QrDecoder::decode_from_file(from_image, use_zbar).await?;
    let entries = match origin {
        Origin::GoogleAuthenticator => {
            QrDecoder::parse_google_auth_export(&content)?
        }
    };
    let config_entries: Vec<ConfigEntry> = entries
        .into_iter()
        .filter_map(|e| {
            let name = e.name.clone();
            let mut entry = ConfigEntry::from(e);
            if prompt {
                print!(
                    "Enter code for {} (or `-` to not include into config): ",
                    name
                );
                io::stdout().flush().ok()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input).ok()?;
                let code = input.trim();

                if code == "-" {
                    return None; // skip this entry
                } else {
                    entry.handle = code.to_owned();
                }
            }

            Some(entry)
        })
        .collect();
    println!("{}", serde_json::to_string_pretty(&config_entries)?);
    Ok(())
}
