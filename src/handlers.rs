use actix_web::{HttpResponse, Responder};
pub mod websocket;

pub async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world from Bear hole!")
}
