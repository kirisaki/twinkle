use std::net::SocketAddr;

// limitation of uUDP
pub const BUF_SIZE: usize = 64 * 1024;
pub const UUID_LEN: usize = 16;

pub type Bytes = Vec<u8>;
pub type UUID = Vec<u8>;

#[derive(Debug)]
pub struct Packet {
    pub dest: SocketAddr,
    pub body: Bytes,
    pub amt: usize,
}

