extern crate twinkled;

use std::sync::Arc;
use std::collections::HashMap;

use futures::future::{try_join};

use tokio::net::UdpSocket;
use tokio::sync::mpsc::channel;
use tokio::sync::Mutex;

use twinkled::types::*;
use twinkled::store::Store;
use twinkled::receiver::Server;
use twinkled::transmitter::Client;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (rxs, txs) = UdpSocket::bind("0.0.0.0:3000").await?.split();
    let (txc, rxc) = channel(1024);
    let store = Store::new();
    let server = Server {sock: rxs, chan: txc, buf: vec![0; BUF_SIZE]};
    let client = Client {sock: txs, chan: rxc, store};

    let _ = try_join(server.run(), client.run()).await?;

    Ok(())
}

