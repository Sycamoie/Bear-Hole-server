// the banker which is responsible for the gameplay
use super::controllers::ws_controller;
use super::game_room::GameRoom;
use super::message::{Connect, Disconnect, WsMessage};

use actix::{
    fut, Actor, ActorContext, ActorFuture, Addr, AsyncContext, ContextFutureSpawner, Handler,
    Running, StreamHandler, WrapFuture,
};
use actix_web_actors::ws;
use log::{error, info};
use std::time::Instant;
use uuid::Uuid;

mod meta {
    use dotenv;
    use std::time::Duration;

    pub fn heartbeat_interval() -> Duration {
        Duration::from_millis(
            dotenv::var("HB_INTERVAL")
                .expect("unable to read HB_INTERVAL")
                .parse::<u64>()
                .expect("unable to parse HB_INTERVAL"),
        )
    }
    pub fn client_timeout() -> Duration {
        Duration::from_secs(
            dotenv::var("CLIENT_TIMEOUT")
                .expect("unable to read CLIENT_TIMEOUT")
                .parse::<u64>()
                .expect("unable to parse CLIENT_TIMEOUT"),
        )
    }
}
pub struct Banker {
    pub room_id: usize,
    pub room_addr: Addr<GameRoom>,
    pub hb: Instant, // heartbeat
    pub id: Uuid,
}

impl Actor for Banker {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);

        let addr = ctx.address();
        self.room_addr
            .send(Connect {
                id: self.id,
                room_id: self.room_id,
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, _, ctx| {
                match res {
                    Ok(_) => (),
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.room_addr.do_send(Disconnect {
            id: self.id,
            room_id: self.room_id,
        });
        Running::Stop
    }
}

impl Banker {
    pub fn new(room_id: usize, room: Addr<GameRoom>) -> Banker {
        Banker {
            room_id,
            room_addr: room,
            hb: Instant::now(),
            id: Uuid::new_v4(),
        }
    }

    /// heartbeat method
    pub fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(meta::heartbeat_interval(), |act, ctx| {
            // if time over the heartbeat
            if Instant::now().duration_since(act.hb) > meta::client_timeout() {
                info!("heartbeat failed");
                act.room_addr.do_send(Disconnect {
                    id: act.id,
                    room_id: act.room_id,
                });
                // stop the connection
                ctx.stop();
                return;
            }
            // else PING
            ctx.ping(b"");
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Banker {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        // info!("WS Message: {:?}", msg);
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            Ok(ws::Message::Continuation(_)) => {
                ctx.stop();
            }
            Ok(ws::Message::Nop) => (),
            Ok(ws::Message::Text(s)) => ws_controller(&self, s),
            Err(e) => error!("unexpected message: {}", e),
        }
    }
}

impl Handler<WsMessage> for Banker {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}
