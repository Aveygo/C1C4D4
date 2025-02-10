use iroh::{endpoint::{Connection, RecvStream, SendStream, VarInt}, Endpoint, PublicKey};
use serde::{Serialize, Deserialize};

use tokio::io::BufReader;
use tokio::io::AsyncReadExt;
use tokio::io;
use tokio::time::{timeout, Duration};
use std::sync::Arc;

use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Debug)]
pub struct Ping { }

#[derive(Serialize, Deserialize, Debug)]
pub struct Pong { }

#[derive(Serialize, Deserialize, Debug)]
pub struct CloseRequest { }

#[derive(Serialize, Deserialize, Debug)]
pub struct PlannedClose { }


#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
pub enum NetworkEvent {
    Ping(Ping),
    Pong(Pong),
    CloseRequest(CloseRequest),
}

#[derive(Debug)]
pub enum NetworkEventError {
    Io(io::Error),
    Json(serde_json::Error),
    IncompleteData,
	Timeout,
	SafeClose
}

pub struct Pipe {
	pub send: SendStream,
	pub recv: RecvStream,
	pub node: PublicKey,
	pub connection: Connection,
	pub expire_epoch: u64,
	pub endpoint: Arc<Endpoint>
}

impl Pipe {
	pub fn new(send:SendStream, recv:RecvStream, node:PublicKey, connection:Connection, endpoint:Arc<Endpoint>) -> Self {

		let start = SystemTime::now();
		let expire_epoch = start
        	.duration_since(UNIX_EPOCH)
        	.expect("Time went backwards")
			.as_secs() + 20u64;

		Pipe{
			send,
			recv,
			node,
			connection,
			expire_epoch,
			endpoint
		}
	}

	// TODO, implement timeouts: nodes can start communicating and just malicously stop 

	pub async fn receive(&mut self) -> Result<NetworkEvent, NetworkEventError> {
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
	
				match serde_json::from_slice::<NetworkEvent>(complete_data) {
					Ok(event) => {
						return Ok(event);
					}
					Err(e) => {
						eprintln!("Failed to deserialize JSON: {:?}", e);
					}
				}
	
				accumulated_data = accumulated_data.split_off(pos + 1);
			}
		}
	
		Err(NetworkEventError::IncompleteData)
	}
	

	pub async fn send(&mut self, event:NetworkEvent) {
		let data = serde_json::to_string(&event).unwrap();
		println!("Sending {:?}", data);
		let data = data.as_bytes();
		let data = [&data, "\n".as_bytes()].concat();
		self.send.write(&data).await.unwrap();

	}

	pub async fn close(&mut self) {
		println!("Closing the connection");
		//self.send.finish().unwrap();
    	//self.connection.closed().await;
		println!("closed the pipe");
		
	}
}

pub struct LogicHandler {
}

impl LogicHandler {
    pub fn new() -> Self {

    	LogicHandler{
    	}
    }

	pub async fn received_ping(&mut self, ping:Ping, pipe:&mut Pipe) -> () {
		println!("received ping {:?}", ping);
		pipe.send(NetworkEvent::Pong(Pong{})).await;
		println!("Sent pong");
	}

	pub async fn received_pong(&mut self, pong:Pong, pipe:&mut Pipe) -> () {
		println!("Received a pong {:?}", pong);
		pipe.send(NetworkEvent::CloseRequest(CloseRequest{})).await;
		pipe.connection.closed().await;
		println!("Sent a close request");


	}

	pub async fn received_close(&mut self, close:CloseRequest, pipe:&mut Pipe) -> () {
		println!("Received a close request {:?}, closing the pipe...", close);
		//
		
		pipe.send.finish().unwrap();
		pipe.connection.close(VarInt::from_u32(200), b"Received close request");
		//pipe.close().await;


		println!("pipe closed");
	}
	
	pub async fn handle(&mut self, mut pipe: Pipe) {
		
		loop {
			let response = pipe.receive().await;
			
			if let Ok(response) = response {

				let action = match response {
					NetworkEvent::Ping(ping) => { self.received_ping(ping, &mut pipe).await },
					NetworkEvent::Pong(pong) => { self.received_pong(pong, &mut pipe).await },
					NetworkEvent::CloseRequest(close) => { 
						self.received_close(close, &mut pipe).await;
						println!("We closed the pipe");
						return;
					}
				};
			} else {
				eprintln!("Pipe encountered: {:?}", response);
				return;
			}

		}
	}


}


