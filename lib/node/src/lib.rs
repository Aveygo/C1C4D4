use config::ConfigLoader;
use iroh::{Endpoint, PublicKey};
use std::sync::Arc;
use log::{info, warn};

use event_handler::{connection::ConnectionLogic, handlers::NetworkEvent, pipe::Pipe};

const CHAT_ALPN: &[u8] = b"pkarr-discovery-demo-chat";

pub struct Node {
    pub endpoint: Arc<Endpoint>,
    pub public_key: PublicKey,
}

pub struct Ping {
    pub destination: PublicKey,
}

pub struct Hearbeat {
    pub destination: PublicKey,
}

pub enum Command {
    Ping(Ping),
    Hearbeat(Hearbeat)
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

        Node {
            endpoint: Arc::new(endpoint),
            public_key: public_key,
        }
        
    }

    pub async fn push_to_thread(&mut self, mut connection:ConnectionLogic) {
        tokio::spawn(async move {
            connection.handle().await;
            info!("Connection stopped");
        });
    }

    pub async fn push(&mut self, destination:PublicKey, event:NetworkEvent) {
        let pipe = self.connect_to_node(destination).await;
        let mut connection = ConnectionLogic::new(pipe);
        connection.pipe.send(event).await;
        self.push_to_thread(connection).await;

    }

    pub async fn connect_to_node(&mut self, node:PublicKey) -> Pipe<NetworkEvent> {
        let connection = self.endpoint.connect(node, CHAT_ALPN).await.unwrap();     
        info!("Connection made with {:?}", node);             
        let (send, recv) = connection.open_bi().await.unwrap();
        Pipe::new(send, recv, node, connection)
    }
   
    pub async fn listen(&mut self) {
        while let Some(incoming) = self.endpoint.accept().await {

            let connecting = match incoming.accept() {
                Ok(connecting) => connecting,
                Err(err) => {
                    warn!("Unstable incoming connection: {err:#}");
                    continue;
                }
            };
            
            let connection = connecting.await.unwrap();
            let node = connection.remote_node_id().unwrap();
            info!("Connection made with {:?}", node);

            let (send, recv) = connection.accept_bi().await.unwrap();

            let pipe:Pipe<NetworkEvent> = Pipe::new(send, recv, node, connection);
            let connection = ConnectionLogic::new(pipe);
            self.push_to_thread(connection).await;
        }
    }



}