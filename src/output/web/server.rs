use actix_web::{get, http::header::{self, Accept}, mime, web, App, HttpResponse, HttpResponseBuilder, HttpServer, Responder};
use crate::{config::secrets::get_secret, load_secrets, totp::{generate_totp, Totp}};

#[get("/list")]
async fn list_entries(secrets_path: web::Data<String>) -> impl Responder {
    let names: Vec<_> = load_secrets(secrets_path.as_str()).entries;
    HttpResponse::Ok().json(names)
}

fn plain_text_response(mut resp: HttpResponseBuilder, totp: Totp) -> HttpResponse {
    resp.content_type("text/plain").body(format!("{}\n", totp.token))
}

fn accept_contains_json(accept: &Accept) -> bool {
    accept.0.iter().any(|qi| {
        let mime = &qi.item;
        mime.type_() == mime::APPLICATION && mime.subtype() == mime::JSON
    })
}

#[get("/code/{code}")]
async fn get_code(secrets_path: web::Data<String>, path: web::Path<String>, accept: Option<web::Header<header::Accept>>) -> impl Responder {
    let code = path.into_inner();

    if let Some(entry) = get_secret(secrets_path.as_str(), code.as_str()) {
        let totp = generate_totp(entry.secret.as_str(), 6, None, None);
        let mut builder = HttpResponse::Ok();
        match accept {
            Some(header) => {
                let header = header.into_inner();
                if accept_contains_json(&header) {
                    builder.json(totp)
                } else {
                    plain_text_response(builder, totp)
                }
            },
            _ => plain_text_response(builder, totp),
        }
    } else {
        HttpResponse::NotFound().body("No matching code found.")
    }
}

// Function to launch the server
pub async fn start_server(bind: &str, port: u16, secrets_path: String) -> std::io::Result<()> {
    println!("Starting server at http://{}:{}", bind, port);

    println!("Secrets will be read from {}", secrets_path);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(secrets_path.clone()))
            .service(list_entries)
            .service(get_code)
    })
    .bind((bind, port))?
    .run()
    .await
}
