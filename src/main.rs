use tokio::net::UdpSocket;


const BUF_SIZE: usize = 64 * 1024;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    {
        let mut socket = UdpSocket::bind("127.0.0.1:3000").await?;

        loop {
            let mut buf = [0; BUF_SIZE];
            match socket.recv_from(&mut buf).await {
                Ok(v) =>
                    tokio::spawn(handler(v, buf)),
                Err(e) => {
                    println!("error: {}", e);
                    continue;
                }
            };
        }
    }
}

async fn handler(pair: (usize, std::net::SocketAddr), buf: [u8; BUF_SIZE]) {
    let (amt, src) = pair;
    if amt < 3 {
        "invalid data";
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
            let cmd = &buf[0];
            let key = &buf[1..pos];
            let value = &buf[pos + 2..amt];
            println!("cmd: {:?}, key: {:?}, val: {:?}", cmd, key, value);
        } else {
            "invalid data";
        }
    }
}
