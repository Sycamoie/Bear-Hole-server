use crate::handlers::websocket::message::{ClientActorMessage, Connect, Disconnect, WsMessage};

use actix::prelude::{Actor, Context, Handler, Recipient};
use log::info;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub struct GameRoom {
    sessions: HashMap<Uuid, Recipient<WsMessage>>,
    rooms: HashMap<usize, HashSet<Uuid>>,
}

impl Default for GameRoom {
    fn default() -> GameRoom {
        // default room
        let mut rooms = HashMap::new();
        rooms.insert(1 as usize, HashSet::new());

        GameRoom {
            sessions: HashMap::new(),
            rooms,
        }
    }
}

impl GameRoom {
    fn send_message(&self, message: &str, dest_id: &Uuid) {
        if let Some(socket_recipient) = self.sessions.get(dest_id) {
            socket_recipient
                .do_send(WsMessage(message.to_owned()))
                .expect("fail sending message")
        } else {
            info!(
                "attempting to send message but couldn't find user id {}.",
                dest_id
            );
        }
    }
}

impl Actor for GameRoom {
    type Context = Context<Self>;
}

/// Handler for connect message.
impl Handler<Connect> for GameRoom {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        info!(
            "inserting {} into room {} where {:?}",
            msg.id, msg.room_id, self.rooms
        );

        // create a room if necessary, and then add the id to it
        self.rooms
            .entry(msg.room_id)
            .or_insert_with(HashSet::new)
            .insert(msg.id);

        // send to everyone in the room that new uuid just joined
        self.rooms
            .get(&msg.room_id)
            .unwrap()
            .iter()
            .filter(|conn_id| *conn_id.to_owned() != msg.id)
            .for_each(|conn_id| self.send_message(&format!("/i connected {}", msg.id), conn_id));

        // store the address
        self.sessions.insert(msg.id, msg.addr);

        // send self your new uuid
        self.send_message(&format!("/i info {}", msg.id), &msg.id);
        info!("room id: {:?} among rooms: {:?}", msg.room_id, self.rooms);
    }
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for GameRoom {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        if self.sessions.remove(&msg.id).is_some() {
            self.rooms
                .get(&msg.room_id)
                .unwrap()
                .iter()
                .filter(|conn_id| *conn_id.to_owned() != msg.id)
                .for_each(|user_id| {
                    self.send_message(&format!("/i disconnected {}", &msg.id), user_id)
                });
            if let Some(lobby) = self.rooms.get_mut(&msg.room_id) {
                if lobby.len() > 1 {
                    lobby.remove(&msg.id);
                } else {
                    //only one in the lobby, remove it entirely
                    self.rooms.remove(&msg.room_id);
                }
            }
        }
    }
}

impl Handler<ClientActorMessage> for GameRoom {
    type Result = ();

    fn handle(&mut self, msg: ClientActorMessage, _: &mut Context<Self>) -> Self::Result {
        // whisper message
        if msg.msg.starts_with("/w") {
            if let Some(id_to) = msg.msg.split(' ').collect::<Vec<&str>>().get(1) {
                self.send_message(&msg.msg, &Uuid::parse_str(id_to).unwrap());
            }
        } else if msg.msg.starts_with("/k") {
        } else {
            self.rooms
                .get(&msg.room_id)
                .unwrap()
                .iter()
                .for_each(|client| self.send_message(&msg.msg, client));
        }
    }
}
