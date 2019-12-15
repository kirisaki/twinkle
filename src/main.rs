#[macro_use] extern crate failure;

use std::net::SocketAddr;
use std::sync::{Arc};
use std::collections::HashMap;
use std::io::{Error, ErrorKind};

use futures::future::{try_join};

use tokio::net::UdpSocket;
use tokio::net::udp::{RecvHalf, SendHalf};
use tokio::sync::mpsc::{Sender, Receiver, channel};
use tokio::sync::Mutex;


// limitation of uUDP
const BUF_SIZE: usize = 64 * 1024;
const UUID_LEN: usize = 16;

type Bytes = Vec<u8>;
type Store = HashMap<Bytes, Bytes>;
type UUID = Vec<u8>;

#[derive(Debug, Fail)]
enum TwinkleError {
    #[fail(display = "parse error")]
    ParseError,
    #[fail(display = "something wrong")]
    SomethingWrong,
}

impl From<TwinkleError> for Error {
    fn from(e: TwinkleError) -> Error {
        std::io::Error::new(ErrorKind::Other, e.to_string())
    }
}


#[derive(Debug)]
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
        if amt < UUID_LEN + 1 {
            e
        } else if amt == UUID_LEN + 1 {
            let req = match body[0] {
                0x01 =>
                    Request::Ping,
                _ => return e
            };
            let uuid = body[1..UUID_LEN+1].to_vec();
            Ok(Instruction{req, uuid, dest})
        } else if amt == UUID_LEN + 2 {
            e
        } else {
            let cmd = body[0];
            let uuid = body[1..UUID_LEN + 2].to_vec();
            let high: usize = From::from(body[UUID_LEN + 1]);
            let low: usize = From::from(body[UUID_LEN + 2]);
            let keylen = high * 256 + low;
            let key = if keylen == 0 {
                vec![]
            } else {
                body[UUID_LEN + 3..UUID_LEN + 3 + keylen].to_vec()
            };
            let val = if UUID_LEN + 3 + keylen == amt {
                vec![]
            } else {
                body[UUID_LEN + 3 + keylen..amt].to_vec()
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
            Ok(Instruction{req, uuid, dest})
        }
    }
}

#[derive(Debug)]
struct Instruction {
    req: Request,
    uuid: UUID,
    dest: SocketAddr,
}

impl Instruction {
    async fn respond(self, s: Arc<Mutex<Store>>) -> Result<(Bytes, SocketAddr), TwinkleError> {
        let mut store = s.lock().await;
        let Instruction{req, uuid, dest} = self;
        let resp = match req {
            Request::Ping => {
                let mut r = vec![1];
                r.append(&mut uuid.clone());
                r
            },
            Request::Get(k) => {
                match store.get(&k) {
                    Some(v) => {
                        let mut r = vec![1];
                        r.append(&mut uuid.clone());
                        r.append(&mut v.clone());
                        r
                    },
                    None => {
                        let mut r = vec![2];
                        r.append(&mut uuid.clone());
                        r
                    },
                }
            },
            Request::Set(k, v) => {
                store.insert(k.clone(), v.clone());
                let mut r = vec![1];
                r.append(&mut uuid.clone());
                r
            },
            Request::Unset(k) => {
                store.remove(&k);
                let mut r = vec![1];
                r.append(&mut uuid.clone());
                r
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



fn serialize(s: Store) -> Option<Bytes> {
    let mut output = vec![];
    for (mut k, mut v) in s {
        match append_bytes(&mut output, &mut k) {
            Some(_) => {},
            None => return None,
        };
        match append_bytes(&mut output, &mut v) {
            Some(_) => {},
            None => return None,
        };
    };
    Some(output)
}

fn append_bytes(bytes: &mut Vec<u8>, data: &mut Vec<u8>) -> Option<()> {
    let l = bytes.len();
    if l > 65535 {
        return None;
    };
    let lb = l.to_be_bytes();
    bytes.append(&mut vec![lb[6], lb[7]]);
    bytes.append(data);
    Some(())
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (rxs, txs) = UdpSocket::bind("0.0.0.0:3000").await?.split();
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
            (b"\x01iiiijjjjkkkkllll".to_vec(), Request::Ping),
            (b"\x02iiiijjjjkkkkllll\x00\x01a".to_vec(), Request::Get(b"a".to_vec())),
            (b"\x03iiiijjjjkkkkllll\x00\x01abc".to_vec(), Request::Set(b"a".to_vec(),b"bc".to_vec())),
            (b"\x04iiiijjjjkkkkllll\x00\x01a".to_vec(), Request::Unset(b"a".to_vec())),
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
