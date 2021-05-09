use super::banker::Banker;
use super::message::ClientActorMessage;
use log::info;

pub fn ws_controller(banker: &Banker, message: String) {
    let _ = banker;
    info!("message received {}", message);
    banker.room_addr.do_send(ClientActorMessage {
        id: banker.id,
        room_id: banker.room_id,
        msg: message,
    });
}
