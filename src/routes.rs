use crate::handlers::hello;

use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg
        // test echo
        .route("/", web::get().to(hello))
        .service(
            web::scope("/api/v1")
                //TODO lock with jwt
                // .wrap(AuthMiddleware)
                //TODO add authentication
                // .service(web::scope("/auth"))
                // room
                .route("/rooms", web::get().to(hello))
                .route("/room/{room_id}", web::get().to(hello))
                .route("/room/{room_id}", web::post().to(hello)),
        );
}
