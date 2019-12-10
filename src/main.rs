use std::net::UdpSocket;
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::sync::{Arc};


const BUF_SIZE: usize = 64 * 1024;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = Arc::new(Mutex::new(HashMap::new()));
    let socket = UdpSocket::bind("127.0.0.1:3001")?;

    loop {
        let c_socket = socket.try_clone()?;
        let mut buf = [0; BUF_SIZE];
        match socket.recv_from(&mut buf) {
            Ok(v) => {
                tokio::spawn(handler(v, buf, store.clone(), c_socket))
            },
            Err(e) => {
                println!("error: {:?}", e);
                continue;
            }
        };
    }
 }

async fn handler(pair: (usize, std::net::SocketAddr),
                 buf: [u8; BUF_SIZE],
                 store: Arc<Mutex<HashMap<Vec<u8>, Vec<u8>>>>,
                 socket: std::net::UdpSocket) -> Result<(), String> {
    let mut store = store.lock().await;
    let (amt, src) = pair;
    match parse_body(&buf, amt) {
        Some((cmd, key, value)) => {
            println!("c: {:?}, k: {:?}, v: {:?}", cmd, key, value);
            let resp = match cmd {
                //ping
                0x00 => vec![0x00],
                //get
                0x01 =>
                    match store.get(key) {
                        Some(v) => {
                            [vec![0x01], key.to_vec(), vec![0x0d, 0x0a], v.to_vec()].concat()
                        },
                        None =>
                            vec![0x02],
                    },
                //set
                0x02 => {
                    let _ = store.insert(key.to_vec(), value.to_vec());
                    vec![0x01]
                },
                //unset
                0x03 => {
                    let _ = store.remove(&key.to_vec());
                    vec![0x01]
                },
                _ => vec![0x02],
            };
            let _ = socket.send_to(&resp, &src);
            return Ok(());
        },
        None =>
            return Err("failed parsing".to_string())
    }
}


fn parse_body<'a>(buf: &'a [u8; BUF_SIZE], amt: usize) -> Option<(u8, &'a [u8], &'a [u8])> {
    if amt >= 3 {
        let cmd = buf[0];
        let high: usize = From::from(buf[1]);
        let low: usize = From::from(buf[2]);
        let keylen = high * 256 + low;
        let key = if keylen == 0 {
            &[]
        } else {
            &buf[3..3+keylen]
        };
        let value = if 3 + keylen == amt {
            &[]
        } else {
            &buf[3+keylen..amt]
        };
        return Some((cmd, key, value));
    } else {
        return None;
    }
}
