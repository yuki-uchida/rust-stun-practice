use anyhow::Result;
use std::sync::Arc;
use stun::message::*;
use tokio::net::UdpSocket;
#[tokio::main]
async fn main() -> Result<()> {
    let conn = Arc::new(UdpSocket::bind("0.0.0.0:3478").await?);
    // println!("{:?}", socket);
    let mut buf = [0u8; 1500];
    loop {
        let (n, addr) = match conn.recv_from(&mut buf).await {
            Ok((n, addr)) => (n, addr),
            Err(err) => {
                break;
            }
        };
        println!("{:?} {:?}", n, addr);
        println!("{:?}", &buf[..n]);

        // check packet type.
        if &buf[..2] != [0, 1] {
            continue;
        }
        println!("received packet's type is STUN packet");
        // このaddrをXOR_MAPPED_ADDRESSにして返す仕組みを作る。
        let message = Message::decode_from_packet(&buf[..n].to_vec()).unwrap();
        println!("{:?}", message);
        // println!("{:?}", message.method);
        // println!("{:?}", message.class);
        if (message.method == METHOD_BINDING) && (message.class == CLASS_REQUEST) {
            println!("received BINDING REQUEST STUN packet.");
            // 特定の処理(?)をしたのちsuccess responseを返す
        }
    }
    Ok(())
}
