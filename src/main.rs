use crate::server::server;

mod handlers;
mod routes;
mod server;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    server().await
}
