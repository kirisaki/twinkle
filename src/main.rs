#[macro_use] extern crate failure;

use tokio::net::UdpSocket;
use tokio::net::udp::{RecvHalf, SendHalf};
use std::collections::HashMap;
use futures::future::{try_join};
use tokio::sync::mpsc::{Sender, Receiver, channel};
use std::net::SocketAddr;
use failure::Error;

// limitation of uUDP
const BUF_SIZE: usize = 64 * 1024;

#[derive(Debug, Fail)]
enum TwinkleError {
    #[fail(display = "parse error")]
    ParseError,
}

struct Packet {
    dest: SocketAddr,
    body: Vec<u8>,
    amt: usize,
}

enum Request {
    Ping,
    Get(Vec<u8>),
    Set(Vec<u8>, Vec<u8>),
    Unset(Vec<u8>),
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

struct Instruction {
    req: Request,
    dest: SocketAddr,
}

struct Server {
    sock: RecvHalf,
    chan: Sender<Packet>,
    buf: Vec<u8>,
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
    store: HashMap<Vec<u8>, Vec<u8>>,
}

impl Client {
    async fn run(self) -> Result<(), std::io::Error> {
        let Client {mut sock, mut chan, mut store} = self;
        while let Some(p) = chan.recv().await {
        };

        Ok(())
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (rxs, txs) = UdpSocket::bind("127.0.0.1:3000").await?.split();
    let (txc, rxc) = channel(1024);
    let server = Server {sock: rxs, chan: txc, buf: vec![0; BUF_SIZE]};
    let client = Client {sock: txs, chan: rxc, store: HashMap::new()};

    let _ = try_join(server.run(), client.run()).await;

    Ok(())
}
