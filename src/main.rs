use tokio::net::UdpSocket;
use tokio::net::udp::{RecvHalf, SendHalf};
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::sync::{Arc};
use futures::future::{try_join};
use tokio::sync::mpsc::{Sender, Receiver, channel};
use std::io::{Error, ErrorKind};

// limitation of uUDP
const BUF_SIZE: usize = 64 * 1024;

struct Instruction {
    dest: std::net::SocketAddr,
}

struct Server {
    sock: RecvHalf,
    chan: Sender<Instruction>,
    buf: Vec<u8>,
}

impl Server {
    async fn run(self) -> Result<(), std::io::Error> {
        let Server {mut sock, mut chan, mut buf} = self;
        loop {
            let (_, src) = sock.recv_from(&mut buf).await?;
            chan.try_send(Instruction{dest: src});
            println!("send");
        }
    }
}

struct Client {
    sock: SendHalf,
    chan: Receiver<Instruction>, 
}

impl Client {
    async fn run(self) -> Result<(), std::io::Error> {
        let Client {mut sock, mut chan} = self;
        while let Some(i) = chan.recv().await {
            sock.send_to(b"\n\nnyaan", &i.dest).await;
            println!("{:?}", i.dest);
            tokio::time::delay_for(std::time::Duration::from_secs(2)).await;
        };

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (rxs, txs) = UdpSocket::bind("127.0.0.1:3000").await?.split();
    let (txc, rxc) = channel(1024);
    let server = Server {sock: rxs, chan: txc,  buf: vec![0; BUF_SIZE]};
    let client = Client {sock: txs, chan: rxc};

    let _ = try_join(server.run(), client.run()).await;

    Ok(())
}
/*
async fn handler(pair: (usize, std::net::SocketAddr),
                 buf: [u8; BUF_SIZE],
                 store: Arc<Mutex<HashMap<Vec<u8>, Vec<u8>>>>,
                 socket: &mut tokio::net::UdpSocket) -> Result<(), String> {
    let mut store = store.lock().await;
    let (amt, src) = pair;
    match parse_body(&buf, amt) {
        Some((cmd, key, value)) => {
            println!("c: {:?}, k: {:?}, v: {:?}", cmd, key, value);
            let resp = match cmd {
                //ping
                0x00 => {
                    println!("ping");
                    vec![0x00,0x02,0x03]
                },
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
    } else if amt == 1 {
        return Some((buf[0], &[], &[]));
    } else {
        return None
    }
}
#[cfg(test)]
mod tests {
    use crate::{parse_body, BUF_SIZE};

    #[test]
    fn test_parse_body() {
        let cases: &[(&[u8], Option<(u8, &[u8], &[u8])>)] = &[
            (&[0, 0, 0], Some((0, &[], &[]))),
            (&[0, 0, 0, 0], Some((0, &[], &[0]))),
            (&[0, 0], None),
            (&[0, 0, 1, 0], Some((0, &[0], &[]))),
            (&[0, 0, 1, 0, 0], Some((0, &[0], &[0]))),
        ];
        for &(received, expected) in cases {
            let mut buf = [0; BUF_SIZE];
            for i in 0..received.len() {
                buf[i] = received[i];
            }
            assert_eq!(parse_body(&buf, received.len()), expected);
        }
    }
}
 */
