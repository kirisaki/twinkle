use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    {
        let mut socket = UdpSocket::bind("127.0.0.1:3000").await?;

        loop {
            let mut buf = [0; 1024 * 64];
            match socket.recv_from(&mut buf).await {
                Ok((amt, _)) => {
                    if amt < 3 {
                        "invalid data";
                    } else {
                        let mut pos :usize = 1;
                        loop {
                            if pos >= amt - 2 {
                                break;
                            }
                            if buf[pos] == 0x0d && buf[pos+1] == 0x0a {
                                break;
                            }
                            pos += 1;
                        }
                        let cmd = &buf[0];
                        let key = &buf[1..pos];
                        let value = &buf[pos + 2..amt];
                        println!("cmd: {:?}, key: {:?}, val: {:?}", cmd, key, value);
                    }
                },
                Err(e) =>
                    eprintln!("error: {}", e)
            }
        }
    }
}
