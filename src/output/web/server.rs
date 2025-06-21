use crate::{
    config::secrets::{ConfigEntryPublic, ConfigFile},
    totp::Totp,
};
use actix_web::{
    App, HttpResponse, HttpResponseBuilder, HttpServer, Responder, get,
    http::header::{self, Accept, ContentType},
    mime, web,
};
use std::sync::Arc;

#[cfg(feature = "http")]
#[get("/list")]
async fn list_entries(secrets_cf: web::Data<Arc<ConfigFile>>) -> impl Responder {
    let result: anyhow::Result<String> = async {
        let (_, secrets) = secrets_cf.load().await?;
        // Convert secrets to their public representation
        let secrets: Vec<ConfigEntryPublic> = secrets.iter().map(|entry| entry.into()).collect();
        let as_string = serde_json::to_string(&secrets)?;
        Ok(as_string)
    }
    .await;
    match result {
        Ok(secrets) => HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(secrets),
        Err(err) => {
            tracing::error!("Failed to load secrets: {}", err);
            HttpResponse::BadRequest().body("Failed to load secrets")
        }
    }
}

fn plain_text_response(mut resp: HttpResponseBuilder, totp: Totp) -> HttpResponse {
    resp.content_type("text/plain")
        .body(format!("{}\n", totp.token))
}

fn accept_contains_json(accept: &Accept) -> bool {
    accept.0.iter().any(|qi| {
        let mime = &qi.item;
        mime.type_() == mime::APPLICATION && mime.subtype() == mime::JSON
    })
}

#[cfg(feature = "http")]
#[get("/code/{code}")]
async fn get_code(
    secrets_cf: web::Data<Arc<ConfigFile>>,
    path: web::Path<String>,
    accept: Option<web::Header<header::Accept>>,
) -> impl Responder {
    let code = path.into_inner();

    let result: anyhow::Result<Totp> = async {
        let (_, secrets) = secrets_cf.load().await?;
        let entry = ConfigFile::get_secret(&secrets, code.as_str())?;
        let totp = Totp::new(entry.secret.as_str(), entry.timestep, entry.digits);
        Ok(totp)
    }
    .await;

    match result {
        Ok(totp) => {
            let mut builder = HttpResponse::Ok();
            match accept {
                Some(header) => {
                    let header = header.into_inner();
                    if accept_contains_json(&header) {
                        builder.json(totp)
                    } else {
                        plain_text_response(builder, totp)
                    }
                }
                _ => plain_text_response(builder, totp),
            }
        }
        Err(_) => HttpResponse::NotFound().body("No matching code found."),
    }
}

// Function to launch the server
#[cfg(feature = "http")]
pub async fn start_server(
    bind: String,
    port: u16,
    secrets_cf: Arc<ConfigFile>,
) -> anyhow::Result<()> {
    tracing::debug!("Secrets will be read from {}", secrets_cf.secrets_path);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Arc::clone(&secrets_cf)))
            .service(list_entries)
            .service(get_code)
    })
    .bind((bind, port))?
    .run()
    .await?;
    Ok(())
}
