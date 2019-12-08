use std::net::UdpSocket;
use std::io::{Error, ErrorKind};

fn main() -> std::io::Result<()> {
    {
        let socket = UdpSocket::bind("127.0.0.1:3000")?;

        loop {
            let mut buf = [0; 1024 * 64];
            match socket.recv_from(&mut buf) {
                Ok((amt, _)) => {
                    let mut pos :usize = 0;
                    loop {
                        println!("{:}", buf[pos]);
                        if pos >= amt {
                            return Err(Error::new(ErrorKind::Other, "invalid data"));
                        }
                        if buf[pos] == 0x0d && buf[pos+1] == 0x0a {
                            break;
                        }
                        pos += 1;
                    }
                    let key = &buf[0..pos];
                    let value = &buf[pos + 2..amt-1];
                    println!("key: {:?}, val: {:?}", key, value);
                },
                Err(e) =>
                    eprintln!("error: {}", e)
            }
        }
    }
}
