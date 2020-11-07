use actix_web::{web, App, HttpServer};
use std::sync::Mutex;
use serde::{Deserialize};

mod attempts;
use attempts::{
    Attempts,
    Code,
    Phone,
    CodeResult,
    PhoneResult,
    PhonesNCodes
};

struct AppState {
    attempts: Mutex<Attempts>
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let data = web::Data::new(AppState {
        attempts: Mutex::new(Attempts::new())
    });

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .route("/code", web::post().to(handle_phone))
            .route("/token", web::post().to(handle_code))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

#[derive(Deserialize)]
struct PhoneData {
    phone: Phone
}

async fn handle_phone(
    data: web::Data<AppState>,
    body: web::Json<PhoneData>
) -> String {

    let mut attempts = match data.attempts.lock() {
        Ok(attempts) => attempts,
        Err(poisoned) => poisoned.into_inner()
    };

    match attempts.get_code(body.phone) {
        PhoneResult::Success(count) => format!("You have {} tries", count),
        PhoneResult::Exists(count, last_attemped_at) => format!("You have {} tries, retry at {:?}", count, last_attemped_at),
        PhoneResult::InvalidPhone => format!("dafaq are you doing")
    }
}

#[derive(Deserialize)]
struct CodeData {
    phone: Phone,
    code: Code
}

async fn handle_code(
    data: web::Data<AppState>,
    body: web::Json<CodeData>
) -> String {
    
    let mut attempts = match data.attempts.lock() {
        Ok(attempts) => attempts,
        Err(poisoned) => poisoned.into_inner()
    };

    match attempts.check_code(body.phone, body.code) {
        CodeResult::InvalidPhone => format!("dafaq are you doing"),
        CodeResult::NotFound => format!("Phone {} not found, you might wanna try /request/code", body.phone),
        CodeResult::Expired => format!("Code request is expired"),
        CodeResult::OutOfAttempts(next_attempt_at) => format!("{:?}", next_attempt_at),
        CodeResult::Invalid(attempts_left) => format!("You have {} attempts left", attempts_left),
        CodeResult::Valid => format!("you lucky dawg")
    }
}