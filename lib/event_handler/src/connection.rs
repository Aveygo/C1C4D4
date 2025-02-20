use crate::handlers::NetworkEvent;
use crate::pipe::{NetworkEventError, Pipe};
use crate::handlers::Handle;

pub struct ConnectionLogic {
    pub pipe: Pipe<NetworkEvent>
}

impl ConnectionLogic {
    pub fn new(pipe: Pipe<NetworkEvent>) -> Self {
        ConnectionLogic { pipe }
    }

    pub async fn handle(&mut self) -> Result<(), NetworkEventError> {
        loop {
            let response = self.pipe.receive().await;

            match response {
                Ok(response) => {
                    response.action(self).await;

                    // Special commands that require stop
                    match response {
                        NetworkEvent::CloseRequest(_) => {return Ok(());},
                        NetworkEvent::CloseResponse(_) => {return Ok(());},
                        _ => {}
                    }
                },
                Err(e) => {return Err(e);}
            }
        }
    }



}