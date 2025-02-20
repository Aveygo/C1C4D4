use tokio::io::{BufReader, AsyncReadExt};
use tokio::time::{timeout, Duration};
use iroh::{endpoint::{Connection, RecvStream, SendStream, VarInt}, PublicKey};
use tokio::io;
use serde::{Deserialize, Serialize};
use log::{info, error};

pub struct Pipe<T> {
    pub send: SendStream,
    pub recv: RecvStream,
    pub node: PublicKey,
    pub connection: Connection,
    _marker: std::marker::PhantomData<T>,
}

#[derive(Debug)]
pub enum NetworkEventError {
    Io(io::Error),
    Json(serde_json::Error),
    IncompleteData,
    Timeout,
    SafeClose,
}

fn debug_bytes(bs: &[u8]) -> String {
    let mut visible = String::new();
    for &b in bs {
        let part: Vec<u8> = std::ascii::escape_default(b).collect();
        visible.push_str(std::str::from_utf8(&part).unwrap());
    }
    visible
}

impl<T> Pipe<T>
where
    T: for<'de> Deserialize<'de> + Serialize + std::fmt::Debug,
{
    pub fn new(send: SendStream, recv: RecvStream, node: PublicKey, connection: Connection) -> Self {
        Pipe {
            send,
            recv,
            node,
            connection,
            _marker: std::marker::PhantomData,
        }
    }

    
    pub async fn receive(&mut self) -> Result<T, NetworkEventError> {
        let mut buffer = vec![0u8; 4096];
        let mut reader = BufReader::new(&mut self.recv);
        let mut accumulated_data = Vec::new();
        
        let timeout_duration = Duration::from_secs(5);
        
        loop {
            let n = timeout(timeout_duration, reader.read(&mut buffer)).await.map_err(|_| NetworkEventError::Timeout)?.map_err(NetworkEventError::Io)?;
            
            if n == 0 {
                if accumulated_data.is_empty() {
                    return Err(NetworkEventError::IncompleteData);
                }
                break;
            }
    
            accumulated_data.extend_from_slice(&buffer[..n]);
    
            if let Some(pos) = accumulated_data.iter().position(|&byte| byte == b'\n') {
                let complete_data = &accumulated_data[..pos];
    
                match serde_json::from_slice::<T>(complete_data) {
                    Ok(event) => {
                        info!("[ {} -> HOST ] Received {:?}", &self.node.to_string()[..6], event);
                        return Ok(event);
                    }
                    Err(e) => {
                        error!("Failed to deserialize JSON from {:?} due to {:?}", self.node, e);
                        error!("Raw received data: {:?}", debug_bytes(complete_data));
                    }
                }
    
                accumulated_data = accumulated_data.split_off(pos + 1);
            }
        }
    
        Err(NetworkEventError::IncompleteData)
    }
    
    pub async fn send(&mut self, event: T) {
        info!("[ HOST -> {} ] Sending {:?}", &self.node.to_string()[..6], event);
        let data = serde_json::to_string(&event).unwrap();
        let data = data.as_bytes();
        let data = [&data, "\n".as_bytes()].concat();
        self.send.write(&data).await.unwrap();
    }

    pub async fn close(&mut self) {
        self.send.finish().unwrap();
		self.connection.close(VarInt::from_u32(200), b"Received close request");
    }

    pub async  fn wait_for_close(&mut self) {
        self.connection.closed().await;
    }


}
