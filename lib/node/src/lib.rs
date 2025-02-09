use std::str::FromStr;
use std::sync::Arc;

use config::ConfigLoader;
use iroh::{Endpoint, NodeId};
use iroh::endpoint::{SendStream, RecvStream};
use pkarr::mainline::rpc::messages::Message;

use tokio::time::{sleep, Duration};

mod events;

pub struct Channel {
    pub node_addr: String, 
    pub writer: SendStream,
    pub reader: RecvStream
}


pub struct Node {
    pub loader: ConfigLoader,
    pub secret_key: iroh::SecretKey,
    pub public_key: iroh::PublicKey,
    pub endpoint: Arc<Endpoint>,
    pub channels: Vec<Channel>,
    pub alpn: String,
    pub listener: Option<tokio::task::JoinHandle<()>>
}

impl Node {
    pub async fn new(config_path:String) -> Self {

        let alpn = "p2psocial/genesis/0.1.0".to_string();
        let loader = ConfigLoader::new(config_path);
        
        println!("Using config {:?}", loader.config.clone());
        let secret:[u8; 32] = hex::decode(loader.config.secret.clone()).expect("could not decode").as_slice().try_into().unwrap();
        let secret_key = iroh::SecretKey::from_bytes(&secret);
        let public_key = secret_key.public();
        println!("public_key {:?}", public_key);

        let discoverable = loader.config.bootstrap_addr.is_none();
        let bootstrap_addr = loader.config.bootstrap_addr.clone();

        let mut discovery = iroh::discovery::pkarr::dht::DhtDiscovery::builder().dht(true);        
        discovery = discovery.n0_dns_pkarr_relay().secret_key(secret_key.clone());
        
        let endpoint = Endpoint::builder()
            .alpns(vec![alpn.as_bytes().to_vec()])
            .secret_key(secret_key.clone())
            .discovery(Box::new(discovery.build().unwrap()))
            .bind()
            .await.unwrap();

        let endpoint = Arc::new(endpoint);
        let endpoint_clone1 = endpoint.clone();
        let endpoint_clone2 = endpoint.clone();


        let mut node = Node {
            loader,
            secret_key,
            public_key,
            endpoint,
            channels: vec![],
            alpn: alpn.clone(),
            listener: None
        };

        

        if !discoverable {
            println!("the node {:?} is the client", public_key);
            tokio::spawn(async move {
                Node::bootstrap(endpoint_clone2, alpn.clone(), bootstrap_addr).await;
            });
        } else {
            println!("the node {:?} is the host", public_key);
            let listener = tokio::spawn(async move {
                Node::listener(endpoint_clone1).await.unwrap();
            });
    
            node.listener = Some(listener);
        }

        node

    }

    pub async fn listener(endpoint: Arc<Endpoint>) -> anyhow::Result<()> {
        println!("Listener created!");

        while let Some(incoming) = endpoint.accept().await {
            let connecting = match incoming.accept() {
                Ok(connecting) => connecting,
                Err(err) => {
                    println!("[WARN] incoming connection may have failed failed: {err:#}");
                    continue;
                }
            };

            tokio::spawn(async move {
                let connection = connecting.await.expect("Connection failed?");
                let remote_node_id = connection.remote_node_id().expect("Could not get remote id");

                
                println!("connected to {:?} waiting for the channel to be opened", remote_node_id);
                let (write, read) = connection.accept_bi().await.expect("Could not accept channel");
                println!("Done waiting for the channel to be opened (SUCCESS!)");


            });
        }

        Ok(())

    }

    pub async fn reader(mut writer:SendStream, mut reader:RecvStream) {

        loop {
            let buffer = reader.read_to_end(4096).await.expect("failed to read pipe");

            // Deserialize the incoming message dynamically based on its type
            let incomming_message: events::Message = serde_json::from_slice(&buffer).expect("failed to decode message");

            match incomming_message {
                events::Message::Ping(req) => {
                    println!("Got a ping ({:?}) Sending out a pong...", req);
                    let outgoing_message = serde_json::to_string(&events::Message::Pong(events::Pong{})).unwrap();
                    writer.write(outgoing_message.as_bytes()).await.expect("Could not send pong!");

                },
                events::Message::Pong(req) => {
                    println!("Got a pong {:?}", req);
                },

            };
        }

    }

    async fn update_config(&mut self) {
        self.loader.config.peers = self.channels.iter().map(|x| x.node_addr.clone()).collect();
        self.loader.dump();
    }

    pub async fn bootstrap(endpoint:Arc<Endpoint>, alpn: String, bootstrap_addr:Option<String>) {
        
        match bootstrap_addr {
            
            Some(boot_addr) => {
                
                let bootstrap_node:NodeId = NodeId::from_str(boot_addr.as_str()).unwrap();
                
                let connection = endpoint.connect(bootstrap_node, alpn.as_bytes()).await.expect("Cannot connect to bootstrap node!");
                
                
                println!("opening the channel");
                let (mut write, read) = connection.open_bi().await.expect("Could not open a channel with the bootstrap node!");
                
                let message = format!("hi! you connected to me. bye bye");
                write.write_all(message.as_bytes()).await.unwrap();



            },
            None => {
                println!("No bootstrap address provided.")
            }
        }

        println!("end of bootstrap");
    }

}