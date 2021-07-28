use anyhow::Result;
use rand::Rng;
use std::fmt;
use std::net::UdpSocket;
use std::{io, str};

#[tokio::main]
async fn main() {
    let socket = UdpSocket::bind("0.0.0.0:34255").expect("couldn't bind to address");
    println!("{:?}", socket);
    let mut buf = [0; 10];
    match socket.recv(&mut buf) {
        Ok(received) => println!("received {} bytes {:?}", received, &buf[..received]),
        Err(e) => println!("recv function failed: {:?}", e),
    }
}
