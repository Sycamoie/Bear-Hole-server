use crate::handlers::websocket;
use crate::routes::routes;

use actix::Actor;
use actix_web::{middleware::Logger, web, App, HttpServer};
use dotenv;
use env_logger::{Builder, Target};

pub async fn server() -> std::io::Result<()> {
    dotenv::dotenv().expect("Error reading .env file");
    Builder::from_default_env().target(Target::Stdout).init();

    let host = dotenv::var("HOST").unwrap();
    let port = dotenv::var("PORT").unwrap();

    //create and spin up a game room
    let game_server = websocket::game_room::GameRoom::default().start();

    HttpServer::new(move || {
        App::new()
            // logger
            .wrap(Logger::default())
            .wrap(Logger::new("%a return %s %b bytes in %D ms"))
            // websocket routes
            .service(
                web::scope("/ws")
                    //websocket route
                    .data(game_server.clone())
                    .route("/{room_id}", web::get().to(websocket::ws_route)),
            )
            // http routes
            .configure(routes)
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}
