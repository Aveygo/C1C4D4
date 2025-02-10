use config::ConfigLoader;
use iroh::{Endpoint, PublicKey};
use std::sync::Arc;

use event_handler::{LogicHandler, NetworkEvent, Pipe};
use event_handler::Ping as NetworkPing;


const CHAT_ALPN: &[u8] = b"pkarr-discovery-demo-chat";

pub struct Node {
    pub endpoint: Arc<Endpoint>,
    pub public_key: PublicKey,
    pub system: LogicHandler
}

pub struct Ping {
    pub destination: PublicKey,
}

pub enum Command {
    Ping(Ping)
}

impl Node {
    pub async fn new(args: ConfigLoader) -> Self {

        let secret:[u8; 32] = hex::decode(args.config.secret.clone()).expect("could not decode").as_slice().try_into().unwrap();
        let secret_key = iroh::SecretKey::from_bytes(&secret);
        let public_key = secret_key.public();

        let discovery = iroh::discovery::pkarr::dht::DhtDiscovery::builder().dht(true)
            .n0_dns_pkarr_relay()
            .secret_key(secret_key.clone())
            .build()
            .unwrap();

        let endpoint = Endpoint::builder()
            .alpns(vec![CHAT_ALPN.to_vec()])
            .secret_key(secret_key)
            .discovery(Box::new(discovery))
            .bind()
            .await
            .unwrap();

        let system = LogicHandler::new();


        Node {
            endpoint: Arc::new(endpoint),
            public_key: public_key,
            system: system
        }
    }

    pub async fn exec(&mut self, target:Command) {
        match target {
            Command::Ping(ping) => {
                println!("Running ping command");

                // Create the pipe that the ping/pong communication will happen over 
                let mut pipe = self.create(ping.destination).await;
                println!("Made pipe! Sending ping...");
                pipe.send(NetworkEvent::Ping(NetworkPing{})).await;
                
                // Let the system handle the response
                self.system.handle(pipe).await;
            }
        }
    }

    pub async fn create(&mut self, node:PublicKey) -> Pipe {
        println!("Trying to connect...");
        let connection = self.endpoint.connect(node, CHAT_ALPN).await.unwrap();                    
        println!("Connection made! Opening channel...");
        let (send, recv) = connection.open_bi().await.unwrap();
        println!("Channel made! Making pipe...");
        Pipe::new(send, recv, node, connection, self.endpoint.clone())
    }

   
    pub async fn listen(&mut self) {
        while let Some(incoming) = self.endpoint.accept().await {
            println!("Incomming connection...");

            let connecting = match incoming.accept() {
                Ok(connecting) => connecting,
                Err(err) => {
                    println!("unstable incoming connection: {err:#}");
                    continue;
                }
            };
            
            let connection = connecting.await.unwrap();
            let node = connection.remote_node_id().unwrap();
            println!("Connection made with {:?}! Opening channel...", node);
            let (send, recv) = connection.accept_bi().await.unwrap();
            println!("Listening....");
            self.system.handle(Pipe::new(send, recv, node, connection, self.endpoint.clone())).await;   

        }
    }



}