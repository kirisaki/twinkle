extern crate twinkled;

use std::path::Path;
use std::fs::File;

use futures::future::try_join3;

use tokio::time::Duration;
use tokio::net::UdpSocket;
use tokio::sync::mpsc::channel;

use twinkled::types::*;
use twinkled::store::Store;
use twinkled::receiver::Server;
use twinkled::transmitter::Client;
use twinkled::snapshooter::Snapshooter;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (rxs, txs) = UdpSocket::bind("0.0.0.0:3000").await?.split();
    let (txc, rxc) = channel(1024);
    let path = Path::new("/tmp/twinkled");
    let store = if path.exists() {
        let reader = File::open(path)?;
        Store::deserialize(reader)
    } else {
        Ok(Store::new())
    }.unwrap(); // TODO: error handling
    let server = Server {sock: rxs, chan: txc, buf: vec![0; BUF_SIZE]};
    let client = Client {sock: txs, chan: rxc, store: store.clone()};
    let snapshooter = Snapshooter{store: store.clone(), path: "/tmp/tinkled", duration: Duration::from_secs(1)};

    let _ = try_join3(
        server.run(),
        client.run(),
        snapshooter.run(),
    ).await?;

    Ok(())
}

