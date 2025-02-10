use serde::{Serialize, Deserialize};

use crate::connection::ConnectionLogic;
pub mod ping;
pub mod pong;
pub mod close_request;
pub mod close_response;
pub mod heartbeat;

pub trait Handle {
    #![allow(async_fn_in_trait)]
    async fn action(&self, connection: &mut ConnectionLogic);
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
pub enum NetworkEvent {
    Ping(ping::Ping),
    Pong(pong::Pong),
    Heartbeat(heartbeat::Heartbeat),
    CloseRequest(close_request::CloseRequest),
    CloseResponse(close_response::CloseResponse),
}


impl Handle for NetworkEvent {
    async fn action(&self, connection: &mut ConnectionLogic) {
        match self {
            NetworkEvent::Ping(ping) => ping.action(connection).await,
            NetworkEvent::Pong(pong) => pong.action(connection).await,
            NetworkEvent::Heartbeat(heart) => heart.action(connection).await,
            NetworkEvent::CloseRequest(close) => { close.action(connection).await; return; }
            NetworkEvent::CloseResponse(close) => { close.action(connection).await; return; }
        }

    }
}