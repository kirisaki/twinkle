#[macro_use] extern crate failure;

use tokio::net::UdpSocket;
use tokio::net::udp::{RecvHalf, SendHalf};
use std::collections::HashMap;
use futures::future::{try_join};
use tokio::sync::mpsc::{Sender, Receiver, channel};
use std::net::SocketAddr;
use failure::Error;
use tokio::sync::Mutex;
use std::sync::{Arc};

// limitation of uUDP
const BUF_SIZE: usize = 64 * 1024;

type Bytes = Vec<u8>;
type Store = HashMap<Bytes, Bytes>;

#[derive(Debug, Fail)]
enum TwinkleError {
    #[fail(display = "parse error")]
    ParseError,
    #[fail(display = "something wrong")]
    SomethingWrong,
}

impl From<TwinkleError> for std::io::Error {
    fn from(e: TwinkleError) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    }
}

struct Packet {
    dest: SocketAddr,
    body: Bytes,
    amt: usize,
}

#[derive(Debug, PartialEq)]
enum Request {
    Ping,
    Get(Bytes),
    Set(Bytes, Bytes),
    Unset(Bytes),
}

impl Packet {
    fn parse(self) -> Result<Instruction, TwinkleError> {
        let Packet {dest, body, amt} = self;
        let e = Err(TwinkleError::ParseError);
        if amt == 0 {
            e
        } else if amt == 1 {
            let req = match body[0] {
                0x01 =>
                    Request::Ping,
                _ => return e
            };
            Ok(Instruction{req, dest})
        } else if amt == 2 {
            e
        } else {
            let cmd = body[0];
            let high: usize = From::from(body[1]);
            let low: usize = From::from(body[2]);
            let keylen = high * 256 + low;
            let key = if keylen == 0 {
                vec![]
            } else {
                body[3..3+keylen].to_vec()
            };
            let val = if 3 + keylen == amt {
                vec![]
            } else {
                body[3+keylen..amt].to_vec()
            };
            let req = match cmd {
                0x02 =>
                    Request::Get(key),
                0x03 =>
                    Request::Set(key, val),
                0x04 =>
                    Request::Unset(key),
                _ =>
                    return e,
            };
            Ok(Instruction{req, dest})
        }
    }
}

#[derive(Debug)]
struct Instruction {
    req: Request,
    dest: SocketAddr,
}

impl Instruction {
    async fn respond(self, s: Arc<Mutex<Store>>) -> Result<(Bytes, SocketAddr), TwinkleError> {
        let mut store = s.lock().await;
        let Instruction{req, dest} = self;
        let resp = match req {
            Request::Ping => vec![1],
            Request::Get(k) => {
                match store.get(&k) {
                    Some(v) => {
                        let mut r = vec![1];
                        r.append(&mut v.clone());
                        r
                    },
                    None =>
                        vec![2]
                }
            },
            Request::Set(k, v) => {
                store.insert(k.clone(), v.clone());
                vec![1]
            },
            Request::Unset(k) => {
                store.remove(&k);
                vec![1]
            },
        };
        Ok((resp, dest))
    }
}

struct Server {
    sock: RecvHalf,
    chan: Sender<Packet>,
    buf: Bytes,
}

impl Server {
    async fn run(self) -> Result<(), std::io::Error> {
        let Server {mut sock, mut chan, mut buf} = self;
        loop {
            let (amt, src) = sock.recv_from(&mut buf).await?;
            let _ = chan.try_send(
                Packet{
                    dest: src,
                    body: buf.to_vec(),
                    amt: amt,
                });
        }
    }
}

struct Client {
    sock: SendHalf,
    chan: Receiver<Packet>,
    store: Arc<Mutex<Store>>,
}

impl Client {
    async fn run(self) -> Result<(), std::io::Error> {
        let Client {mut sock, mut chan, store} = self;
        while let Some(p) = chan.recv().await {
            let (resp, dest) = p.parse()?.respond(store.clone()).await?;
            sock.send_to(&resp, &dest).await?;
        };

        Ok(())
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (rxs, txs) = UdpSocket::bind("127.0.0.1:3000").await?.split();
    let (txc, rxc) = channel(1024);
    let server = Server {sock: rxs, chan: txc, buf: vec![0; BUF_SIZE]};
    let client = Client {sock: txs, chan: rxc, store: Arc::new(Mutex::new(HashMap::new()))};

    let _ = try_join(server.run(), client.run()).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{Packet, Request};
    use std::net::SocketAddr;
    #[test]
    fn test_parse_success() {
        let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
        let cases = vec![
            (b"\x01", Request::Ping),
        ];
        for (received, expected) in cases {
            let packet = Packet{
                dest: addr,
                body: received.to_vec(),
                amt: received.len()
            };
            let result = packet.parse().unwrap();
            assert_eq!(result.req, expected);
        }
    }
}
