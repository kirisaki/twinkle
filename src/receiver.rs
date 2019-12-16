use std::sync::{Arc};
use std::net::SocketAddr;
use tokio::sync::Mutex;
use tokio::net::udp::RecvHalf;
use tokio::sync::mpsc::Sender;

use crate::types::*;
use crate::store::*;
use crate::errors::*;

#[derive(Debug, PartialEq)]
enum Request {
    Ping,
    Get(Bytes),
    Set(Bytes, Bytes),
    Unset(Bytes),
}


pub struct Server {
    pub sock: RecvHalf,
    pub chan: Sender<Packet>,
    pub buf: Bytes,
}

impl Server {
    pub async fn run(self) -> Result<(), std::io::Error> {
        let Server {mut sock, mut chan, mut buf} = self;
        loop {
            let (amt, dest) = sock.recv_from(&mut buf).await?;
            let body = buf.to_vec();
            let _ = chan.try_send(Packet{dest, body, amt});
        }
    }
}


impl Packet {
    pub fn parse(self) -> Result<Instruction, TwinkleError> {
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
pub struct Instruction {
    req: Request,
    uuid: UUID,
    dest: SocketAddr,
}

impl Instruction {
    pub async fn respond(self, s: Store) -> Result<(Bytes, SocketAddr), TwinkleError> {
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

#[cfg(test)]
mod tests {
    use crate::receiver::{Packet, Request};
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
