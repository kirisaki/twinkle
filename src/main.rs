use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::sync::{Arc};


const BUF_SIZE: usize = 64 * 1024;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut socket = UdpSocket::bind("127.0.0.1:3000").await?;
    let store = Arc::new(Mutex::new(0));

    loop {
        let mut buf = [0; BUF_SIZE];
        match socket.recv_from(&mut buf).await {
            Ok(v) => {
                tokio::spawn(handler(v, buf, store.clone()))
            },
            Err(e) => {
                println!("error: {:?}", e);
                continue;
            }
        };
    }
 }

async fn handler(pair: (usize, std::net::SocketAddr), buf: [u8; BUF_SIZE], store: Arc<Mutex<i64>>) {
    let mut store = store.lock().await;
    let (amt, src) = pair;
    match parse_body(&buf, amt) {
        Some((cmd, key, value)) => 
            println!("cmd: {:?}, key: {:?}, value: {:?}", cmd, key, value),
        None =>
            println!("nyaan...")
    }
}

fn parse_body<'a>(buf: &'a [u8; BUF_SIZE], amt: usize) -> Option<(u8, &'a [u8], &'a [u8])> {
    if amt < 3 {
        return None;
    } else {
        let mut pos :usize = 1;
        loop {
            if pos + 1 >= amt  {
                break;
            }
            if buf[pos] == 0x0d && buf[pos+1] == 0x0a {
                break;
            }
            pos += 1;
        }
        if pos <= amt - 2 {
            let cmd = buf[0];
            let key = &buf[1..pos];
            let value = &buf[pos + 2..amt];
            return Some((cmd, key, value));
        } else {
            return None;
        }
    }
}
