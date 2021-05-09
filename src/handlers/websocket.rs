use actix::Addr;
use actix_web::{web, Error};
use actix_web_actors::ws;

pub mod banker;
pub mod controllers;
pub mod game_room;
pub mod message;

pub async fn ws_route(
    req: web::HttpRequest,
    stream: web::Payload,
    path: web::Path<usize>,
    srv: web::Data<Addr<game_room::GameRoom>>,
) -> Result<web::HttpResponse, Error> {
    let room_id = path.into_inner();
    // let room_id = Uuid::from_str("62ed68a3-7128-47a6-8378-ac38a2ef3611").unwrap();
    // let room_id = Uuid::new_v4();

    ws::start(
        banker::Banker::new(room_id, srv.get_ref().clone()),
        &req,
        stream,
    )
}
