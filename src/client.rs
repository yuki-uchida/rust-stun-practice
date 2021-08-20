use std::net::UdpSocket;

use crate::message::*;
use crate::xor_addr::*;

async fn stun_request(msg: Vec<u8>, remote_address: &str) -> Vec<u8> {
    println!("{:?}, {:}", msg, remote_address);
    let socket = UdpSocket::bind("0.0.0.0:34254").expect("couldn't bind to address");
    println!("{:?}", socket);
    socket
        .connect(remote_address)
        .expect("couldn't connect to address");
    socket.send(&msg).expect("couldn't send message");
    let mut buf = [0; 100];
    let mut receive_bytes = 0;
    match socket.recv(&mut buf) {
        Ok(received) => receive_bytes = received,
        Err(e) => println!("recv function failed: {:?}", e),
    }
    println!("{:?}", &buf);
    let xor_addr = &buf[..receive_bytes].to_vec();
    return xor_addr.to_vec();
}

pub async fn get_global_ip(remote_address: &String) -> String {
    let method = METHOD_BINDING;
    let class = CLASS_REQUEST;
    let mut stun_request_message = Message::new(method, class);
    let message = stun_request_message.encode_to_packet();
    let response = stun_request(message, remote_address).await;
    // 受け取ったbytes数の内、20bytesはheaderなので、残りを読めば良い。
    let xor_addr =
        XorMappedAddress::new(response[20..].to_vec(), stun_request_message.transaction_id);
    // xor_addr.get_from(&msg)?;
    println!("{:?}", &xor_addr);
    let global_ip_port = xor_addr.ip.to_string() + ":" + &xor_addr.port.to_string();
    return global_ip_port;
}
