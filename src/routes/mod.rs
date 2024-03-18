use actix_web::{get, HttpResponse, Responder};

#[get("/CheckHealth")]
pub async fn check_health() -> impl Responder {
    HttpResponse::Ok()
}
