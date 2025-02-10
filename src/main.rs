use std::str::FromStr;
use clap::Parser;
use config::ConfigLoader;
use node;


#[derive(Parser)]
struct Args {
    src: String
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    
    let configloader = ConfigLoader::new(args.src.to_string());
    

    match configloader.config.bootstrap_addr.clone() {
        Some(boot) => {
            let mut node = node::Node::new(configloader).await;
            let public_key = iroh::PublicKey::from_str(&boot).unwrap();            
            node.exec(node::Command::Ping(node::Ping{destination: public_key})).await;
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