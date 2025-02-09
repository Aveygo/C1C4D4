use std::{io::Read, str::FromStr};

use clap::Parser;
use iroh::{Endpoint, NodeId, PublicKey};
use tracing::warn;
use url::Url;

use std::time::Instant;
use sha2::{Sha256, Sha512, Digest};
use pkarr::{dns, Keypair, PkarrClient, Result, SignedPacket};
use std::path::Path;

use config;
use node::Node;
use tokio::time::{sleep, Duration};



#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    println!("Starting boot");
    let boot = Node::new("configs/boot.json".to_string()).await;
    let user = Node::new("configs/user.json".to_string()).await;


    match user.listener {
        Some(listener) => listener.await.unwrap(),
        None => {}
    }

    match boot.listener {
        Some(listener) => listener.await.unwrap(),
        None => {}
    }

    
    

    Ok(())
}