//! An example chat application using the iroh endpoint and
//! pkarr node discovery.
//!
//! Starting the example without args creates a server that publishes its
//! address to the DHT. Starting the example with a node id as argument
//! looks up the address of the node id in the DHT and connects to it.
//!
//! You can look at the published pkarr DNS record using <https://app.pkarr.org/>.
//!
//! To see what is going on, run with `RUST_LOG=iroh_pkarr_node_discovery=debug`.
use std::str::FromStr;

use clap::Parser;
use iroh::{Endpoint, NodeId};
use tokio::io::AsyncWriteExt;
use tracing::warn;
use url::Url;
use tokio::net::unix::pipe::pipe;
use hex;
use std::{thread, time::Duration};

use config::ConfigLoader;

const CHAT_ALPN: &[u8] = b"pkarr-discovery-demo-chat";



fn build_discovery() -> iroh::discovery::pkarr::dht::Builder {
    let builder = iroh::discovery::pkarr::dht::DhtDiscovery::builder().dht(true);
    builder.n0_dns_pkarr_relay()
}

async fn chat_server(args: ConfigLoader) -> anyhow::Result<()> {
    let secret:[u8; 32] = hex::decode(args.config.secret.clone()).expect("could not decode").as_slice().try_into().unwrap();
    let secret_key = iroh::SecretKey::from_bytes(&secret);
    
    let discovery = build_discovery()
        .secret_key(secret_key.clone())
        .build()?;

    let endpoint = Endpoint::builder()
        .alpns(vec![CHAT_ALPN.to_vec()])
        .secret_key(secret_key)
        .discovery(Box::new(discovery))
        .bind()
        .await?;

        while let Some(incoming) = endpoint.accept().await {
        let connecting = match incoming.accept() {
            Ok(connecting) => connecting,
            Err(err) => {
                warn!("incoming connection failed: {err:#}");
                continue;
            }
        };
        tokio::spawn(async move {
            let connection = connecting.await?;
            let remote_node_id = connection.remote_node_id()?;
            println!("got connection from {}", remote_node_id);
            // just leave the tasks hanging. this is just an example.
            let (mut writer, mut reader) = connection.accept_bi().await?;
            
            let buffer = reader.read_to_end(100).await.unwrap();
            println!("response: {:?}", buffer);

            writer.finish().unwrap();
            connection.closed().await;

            anyhow::Ok(())
        });
    }
    Ok(())
}

async fn chat_client(args: ConfigLoader) -> anyhow::Result<()> {
    let remote_node_id:NodeId = NodeId::from_str(args.config.bootstrap_addr.unwrap().as_str()).unwrap();
    let secret_key = iroh::SecretKey::generate(rand::rngs::OsRng);
    // note: we don't pass a secret key here, because we don't need to publish our address, don't spam the DHT
    let discovery = build_discovery()
        .secret_key(secret_key.clone())
        .build()?;
    let endpoint = Endpoint::builder()
        .alpns(vec![CHAT_ALPN.to_vec()])
        .secret_key(secret_key)
        .discovery(Box::new(discovery))
        .bind()
        .await?;
    
    let connection = endpoint.connect(remote_node_id, CHAT_ALPN).await?;
    
    println!("connected to {}", remote_node_id);
    
    let (mut writer, mut reader) = connection.open_bi().await?;

    let mut reader: &[u8] = b"hello";
    writer.write_all(reader).await.unwrap();
    writer.finish().unwrap();
    connection.closed().await;

    Ok(())
}

#[derive(Parser)]
struct Args {
    src: String
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    let configloader = ConfigLoader::new(args.src);
    
    match configloader.config.bootstrap_addr.clone() {
        Some(boot) => {
            chat_client(configloader).await.unwrap();
        },  
        None => {
            chat_server(configloader).await.unwrap();
        }
    }
    

    thread::sleep(Duration::from_secs(4000));

    Ok(())
}