use log::warn;
use crate::handlers::NetworkEvent;
use crate::pipe::Pipe;
use crate::handlers::Handle;

pub struct ConnectionLogic {
    pub pipe: Pipe<NetworkEvent>
}

impl ConnectionLogic {
    pub fn new(pipe: Pipe<NetworkEvent>) -> Self {
        ConnectionLogic { pipe }
    }

    pub async fn handle(&mut self) {
        loop {
            let response = self.pipe.receive().await;
            if let Ok(response) = response {
                
                response.action(self).await;

                // Special commands that require stop
                match response {
                    NetworkEvent::CloseRequest(_) => {return;},
                    NetworkEvent::CloseResponse(_) => {return;},
                    _ => {}
                }
                
            } else {
                warn!("Pipe encountered error: {:?}", response);
                return;
            }
        }
    }



}