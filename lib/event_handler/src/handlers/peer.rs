use serde::{Serialize, Deserialize};

use crate::handlers::{Handle, NetworkEvent};
use crate::connection::ConnectionLogic;

#[derive(Serialize, Deserialize, Debug)]
pub struct PostRequest {

}

#[derive(Serialize, Deserialize, Debug)]
pub struct PostResponse {

}

#[derive(Serialize, Deserialize, Debug)]
pub struct SecondaryPeerRequest {

}


#[derive(Serialize, Deserialize, Debug)]
pub struct SecondaryPeerResponse {

}

impl Handle for PostRequest {
    async fn action(&self, connection: &mut ConnectionLogic) {

        
    }
}