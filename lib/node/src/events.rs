use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Ping { }

#[derive(Serialize, Deserialize, Debug)]
pub struct Pong { }


#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Message {
    Ping(Ping),
    Pong(Pong),
}