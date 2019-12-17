extern crate twinkled;

use std::path::Path;
use std::fs::File;
use std::env::var;

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
    let host_port = match var("TWINKLE_HOST_WITH_PORT") {
        Ok(v) => v,
        Err(_) => {
            "127.0.0.1".to_string()
        },
    };
    let db_path = match var("TWINKLE_SNAPSHOT_DB_PATH") {
        Ok(v) => v,
        Err(_) => {
            "./twinkle.db".to_string()
        },
    };
    let duration = match var("TWINKLE_SNAPSHOT_DURATION_SECONDS") {
        Ok(v) => match v.parse::<u64>() {
            Ok(v) => Duration::from_secs(v),
            Err(_) => panic!("invalid duration"),
        },
        Err(_) => {
           Duration::from_secs(3)
        },
    };


    let (rxs, txs) = UdpSocket::bind(host_port).await?.split();
    let (txc, rxc) = channel(1024); // TODO: error handling when a channel overflows
    let path = Path::new(&db_path);
    let store = if path.exists() {
        let reader = File::open(path)?;
        Store::deserialize(reader)
    } else {
        Ok(Store::new())
    }.unwrap(); // TODO: error handling
    let server = Server {sock: rxs, chan: txc, buf: vec![0; BUF_SIZE]};
    let client = Client {sock: txs, chan: rxc, store: store.clone()};
    let snapshooter = Snapshooter {store: store.clone(), path: db_path, duration: duration};

    let _ = try_join3(
        server.run(),
        client.run(),
        snapshooter.run(),
    ).await?;

    Ok(())
}

