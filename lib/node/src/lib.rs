use config::db::NodeDB;
use config::db::identity::Identity;

use iroh::{Endpoint, PublicKey};
use std::sync::Arc;
use log::{info, warn};

use event_handler::{connection::ConnectionLogic, handlers::NetworkEvent, pipe::Pipe};

const CHAT_ALPN: &[u8] = b"pkarr-discovery-demo-chat";

pub struct Node {
    pub endpoint: Arc<Endpoint>,
    pub public_key: PublicKey,
}

impl Node {
    pub async fn new() -> Self {
        
        /*
            TODO, I am feeling sick, so i might leave this project for a sec
            for future me, I just finished lib/config, and main.rs will be broken, but the logic should be ready to integrate with my custom event_handler
            I wrote peer.rs before I redesigned the protocol, but it is roughly how it should be structured

            You need to create an arc for NodeDB so it can be passed to each generated pipe
            From the db, the main functions you need to worry about are in trust_request
            You also need to write manual logic for the bootstrap node to accept connections without trusting first (otherwise the bootstrap node will reject all new connections because a trust request is not possible to make without receiving a post first)
            The DB also assumes you manually trust the user and the bootstrap node, so make sure you do that 
            
            The protocol should have the messages:
                - Post
                - TrustRequest
                - TrustResponse
                - TrustBootstrap
            
            The user should be able to:
                - FetchPosts
                - PromotePost
                - DemotePost

            You will need to consider the case where you cannot connect to a given peer (demote?)
            You could also do something where the node does not construct a pipe if the connecting peer scores too low. 

            Anyways, hope you feel better.
        */
        let db = NodeDB::new("db").unwrap();


        let raw_secret = db.get_identity().unwrap();
        let secret:[u8; 32] = hex::decode(raw_secret.private_key.clone()).expect("could not decode").as_slice().try_into().unwrap();
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
            let r = connection.handle().await;
            
            match r {
                Ok(_r) => info!("Connection stopped {:?}", connection.pipe.node),
                Err(e) => warn!("Connection stopped {:?} with error {:?}", connection.pipe.node, e)
            }
            
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