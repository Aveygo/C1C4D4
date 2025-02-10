use std::str::FromStr;
use clap::Parser;
use config::ConfigLoader;
use node;
use env_logger::Builder;
use log::{self, info};

use event_handler::handlers::{NetworkEvent, ping};

#[derive(Parser)]
struct Args {
    src: String
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Builder::from_env(env_logger::Env::new().default_filter_or("p2psocial=info,event_handler=info,node=info"))
        .init();

    info!("start");

    
    let args = Args::parse();

    
    let configloader = ConfigLoader::new(args.src.to_string());
    

    match configloader.config.bootstrap_addr.clone() {
        Some(boot) => {
            let mut node = node::Node::new(configloader).await;
            let public_key = iroh::PublicKey::from_str(&boot).unwrap();            
            node.push(public_key, NetworkEvent::Ping(ping::Ping {  })).await;
            
            node.listen().await;
            
            
        },  
        None => {
            let mut boot = node::Node::new(configloader).await;
            boot.listen().await;
        }
    }
    

    //thread::sleep(Duration::from_secs(4000));

    Ok(())
}