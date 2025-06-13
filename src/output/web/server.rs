use crate::{config::secrets::get_secret, load_secrets, totp::Totp};
use actix_web::{
    get,
    http::header::{self, Accept},
    mime, web, App, HttpResponse, HttpResponseBuilder, HttpServer, Responder,
};
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(feature = "http")]
#[get("/list")]
async fn list_entries(secrets_path: web::Data<Arc<Mutex<String>>>) -> impl Responder {
    let secrets_path = secrets_path.lock().await;
    tracing::debug!("Listing entries from secrets at: {}", secrets_path);
    let secrets_path = secrets_path.as_str();
    match load_secrets(secrets_path) {
        Ok(secrets) => {
            tracing::debug!("Loaded {} entries from secrets", secrets.entries.len());
            HttpResponse::Ok().json(secrets.entries)
        }
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
    secrets_path: web::Data<Arc<Mutex<String>>>,
    path: web::Path<String>,
    accept: Option<web::Header<header::Accept>>,
) -> impl Responder {
    let code = path.into_inner();

    let secrets_path = secrets_path.lock().await;
    match get_secret(secrets_path.as_str(), code.as_str()) {
        Ok(entry) => {
            let totp = Totp::new(entry.secret.as_str(), entry.timestep, entry.digits);
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
    secrets_path: Arc<Mutex<String>>,
) -> std::io::Result<()> {
    tracing::debug!("Secrets will be read from {secrets_path}");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(secrets_path.clone()))
            .service(list_entries)
            .service(get_code)
    })
    .bind((bind, port))?
    .run()
    .await;
    Ok(())
}
