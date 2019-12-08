use std::net::UdpSocket;

fn main() -> std::io::Result<()> {
    {
        let socket = UdpSocket::bind("127.0.0.1:34254")?;

        loop {
            let mut buf = [0; 1];
            let (amt, src) = socket.recv_from(&mut buf)?;
            
            let buf = &mut buf[..amt];
            buf.reverse();
            socket.send_to(buf, &src)?;
        }      
    }
}
