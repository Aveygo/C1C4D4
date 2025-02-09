use std::str::FromStr;
use std::sync::Arc;

use config::ConfigLoader;
use iroh::{Endpoint, NodeId};
use iroh::endpoint::{SendStream, RecvStream};
use pkarr::mainline::rpc::messages::Message;

use tokio::time::{sleep, Duration};
use iroh::endpoint::Connection;
mod events;

const CHAT_ALPN: &[u8] = b"pkarr-discovery-demo-chat";


fn build_discovery() -> iroh::discovery::pkarr::dht::Builder {
    let builder = iroh::discovery::pkarr::dht::DhtDiscovery::builder().dht(true);
    builder.n0_dns_pkarr_relay()
}

pub async fn chat_client(configloader:ConfigLoader) -> anyhow::Result<()> {
    let remote_node_id = NodeId::from_str(configloader.config.bootstrap_addr.ok_or("no bootstrap").unwrap().as_str()).unwrap();
    
    let secret_key = iroh::SecretKey::generate(rand::rngs::OsRng);
    let node_id = secret_key.public();
    // note: we don't pass a secret key here, because we don't need to publish our address, don't spam the DHT
    let discovery = build_discovery().build()?;
    // we do not need to specify the alpn here, because we are not going to accept connections
    let endpoint = Endpoint::builder()
        .secret_key(secret_key)
        .discovery(Box::new(discovery))
        .bind()
        .await?;
    println!("We are {} and connecting to {}", node_id, remote_node_id);
    let connection = endpoint.connect(remote_node_id, CHAT_ALPN).await?;
    println!("connected to {}", remote_node_id);
    let (mut writer, mut reader) = connection.open_bi().await?;
    let _copy_to_stdout =
        tokio::spawn(async move { tokio::io::copy(&mut reader, &mut tokio::io::stdout()).await });
    let _copy_from_stdin =
        tokio::spawn(async move { tokio::io::copy(&mut tokio::io::stdin(), &mut writer).await });
    _copy_to_stdout.await??;
    _copy_from_stdin.await??;
    Ok(())
}

async fn chat_server(configloader:ConfigLoader) -> anyhow::Result<()> {
    let secret_key = iroh::SecretKey::generate(rand::rngs::OsRng);
    let node_id = secret_key.public();
    let discovery = build_discovery()
        .secret_key(secret_key.clone())
        .build()?;
    let endpoint = Endpoint::builder()
        .alpns(vec![CHAT_ALPN.to_vec()])
        .secret_key(secret_key)
        .discovery(Box::new(discovery))
        .bind()
        .await?;
    let zid = pkarr::PublicKey::try_from(node_id.as_bytes())?.to_z32();
    println!("Listening on {}", node_id);
    println!("pkarr z32: {}", zid);
    println!("see https://app.pkarr.org/?pk={}", zid);
    while let Some(incoming) = endpoint.accept().await {
        let connecting = match incoming.accept() {
            Ok(connecting) => connecting,
            Err(err) => {
                println!("incoming connection failed: {err:#}");
                // we can carry on in these cases:
                // this can be caused by retransmitted datagrams
                continue;
            }
        };
        tokio::spawn(async move {
            let connection = connecting.await?;
            let remote_node_id = connection.remote_node_id()?;
            println!("got connection from {}", remote_node_id);
            // just leave the tasks hanging. this is just an example.
            let (mut writer, mut reader) = connection.accept_bi().await?;
            let _copy_to_stdout = tokio::spawn(async move {
                tokio::io::copy(&mut reader, &mut tokio::io::stdout()).await
            });
            let _copy_from_stdin =
                tokio::spawn(
                    async move { tokio::io::copy(&mut tokio::io::stdin(), &mut writer).await },
                );
            anyhow::Ok(())
        });
    }
    Ok(())
}
