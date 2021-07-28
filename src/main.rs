pub mod xor_addr;
// use crate::xor_addr;

use anyhow::Result;
use rand::Rng;
use std::fmt;
use std::net::UdpSocket;
use std::{io, str};

// const REMOTE_ADDRESS: &str = "stun.l.google.com:19302";
const REMOTE_ADDRESS: &str = "142.250.21.127:19302";
// const REMOTE_ADDRESS: &str = "0.0.0.0:3478";

pub(crate) const MAGIC_COOKIE: u32 = 0x2112A442; // 32bit = 4bytes
const ATTRIBUTE_HEADER_SIZE: usize = 4;
const MESSAGE_HEADER_SIZE: usize = 20; // 160bit = 20bytes
const TRANSACTION_ID_SIZE: usize = 12; // 96bit = 12 bytes
const DEFAULT_RAW_CAPACITY: usize = 120; //960bit = 120bytes

pub trait Setter {
    fn add_to(&self, m: &mut Message) -> Result<()>;
}
// Getter parses attribute from *Message.
pub trait Getter {
    fn get_from(&mut self, m: &Message) -> Result<()>;
}

pub struct Message {
    method: Method,
    class: MethodClass,
    transaction_id: [u8; TRANSACTION_ID_SIZE],
}
impl Message {
    fn new(method: Method, class: MethodClass) -> Self {
        let mut random_transaction_id = [0u8; TRANSACTION_ID_SIZE];
        rand::thread_rng().fill(&mut random_transaction_id);
        // println!("{:?}", random_transaction_id);
        Message {
            method: method,
            class: class,
            transaction_id: random_transaction_id,
        }
    }
    fn build(&mut self) -> Vec<u8> {
        let mut raw = Vec::with_capacity(DEFAULT_RAW_CAPACITY);
        raw.extend_from_slice(&[0; MESSAGE_HEADER_SIZE]);
        //|0|0|TTTTTTTTTTTTTT|LLLLLLLLLLLLLLLL|
        //|  CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC  |
        //|            Transaction ID         |
        //|            Transaction ID         |
        // 00を埋める
        // 1,2byte目 STUN Message Typeを埋める
        let stun_message_type = self.build_message_type().to_be_bytes();
        raw[..2].copy_from_slice(&stun_message_type);
        // 3,4byte目 Message Lengthを埋める
        let stun_message_length = self.build_message_length().to_be_bytes();
        raw[2..4].copy_from_slice(&stun_message_length);
        // 5~8byte目 Magic Cookieを埋める
        raw[4..8].copy_from_slice(&MAGIC_COOKIE.to_be_bytes());
        // 9~20byte目 Transaction IDを埋める
        raw[8..20].copy_from_slice(&self.transaction_id);

        // Attributes
        return raw;
    }
    // 先頭2bitは00で始まることは決まっているので、それ以外の14bitを埋める。
    fn build_message_type(&mut self) -> u16 {
        //	 0                 1
        //	 2  3  4 5 6 7 8 9 0 1 2 3 4 5
        //	+--+--+-+-+-+-+-+-+-+-+-+-+-+-+
        //	|M |M |M|M|M|C|M|M|M|C|M|M|M|M|
        //	|11|10|9|8|7|1|6|5|4|0|3|2|1|0|
        //	+--+--+-+-+-+-+-+-+-+-+-+-+-+-+

        const RIGHT_BIT: u16 = 0xf; // 0b0000000000001111
        const CENTOR_BIT: u16 = 0x70; // 0b0000000001110000
        const LEFT_BIT: u16 = 0xf80; // 0b0000111110000000
        const METHOD_CENTOR_SHIFT: u16 = 1;
        const METHOD_LEFT_DSHIFT: u16 = 2;
        const CLASS_LEFT_BIT: u16 = 0x1; // 0b01
        const CLASS_RIGHT_BIT: u16 = 0x2; // 0b10
        const CLASS_LEFT_SHIFT: u16 = 7;
        const CLASS_RIGHT_SHIFT: u16 = 4;

        // method
        let method = self.method.0 as u16;
        let right_bit = method & RIGHT_BIT;
        let centor_bit = (method & CENTOR_BIT) << METHOD_CENTOR_SHIFT;
        let left_bit = (method & LEFT_BIT) << METHOD_LEFT_DSHIFT;
        let method = left_bit + centor_bit + right_bit;
        println!(
            "method_left_bit: {}, method_centor_bit: {}, method_right_bit: {}, => method: {}",
            left_bit, centor_bit, right_bit, method
        );
        // class
        let class = self.class.0 as u16;
        let c1 = (class & CLASS_LEFT_BIT) << CLASS_LEFT_SHIFT;
        let c0 = (class & CLASS_RIGHT_BIT) << CLASS_RIGHT_SHIFT;
        println!(
            "c1(class_left_bit): {:?}, c0(class_right_bit): {:?}",
            c1, c0
        );
        let class = c1 + c0;
        let message_type_bytes = method + class;
        return message_type_bytes;
    }
    fn build_message_length(&mut self) -> u16 {
        //TODO: impl attributes and message_length
        let stun_message_length = 0 as u16;
        return stun_message_length;
    }
}
impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "method={} class={} transaction_id={:?}",
            self.method, self.class, self.transaction_id
        )
    }
}

#[derive(PartialEq, Eq)]
pub struct Method(u16);
const METHOD_BINDING: Method = Method(0x001);
const METHOD_ALLOCATE: Method = Method(0x003);
impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match *self {
            METHOD_BINDING => "METHOD_BINDING",
            METHOD_ALLOCATE => "METHOD_ALLOCATE",
            _ => "unknown method",
        };
        write!(f, "{}", s)
    }
}

#[derive(PartialEq, Eq)]
pub struct MethodClass(u8);
const CLASS_REQUEST: MethodClass = MethodClass(0x00); // 0b00: request
const CLASS_INDICATION: MethodClass = MethodClass(0x01); // 0b01: indication
const CLASS_SUCCESS: MethodClass = MethodClass(0x10); // 0b10: success
const CLASS_ERROR: MethodClass = MethodClass(0x11); // 0b11: error
impl fmt::Display for MethodClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match *self {
            CLASS_REQUEST => "CLASS_REQUEST",
            _ => "unknown class",
        };
        write!(f, "{}", s)
    }
}

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
    let xor_addr = &buf[..receive_bytes].to_vec();
    println!("{:?}", &buf);
    return xor_addr.to_vec();
}

async fn get_global_ip() -> String {
    let method = METHOD_BINDING;
    let class = CLASS_REQUEST;
    let mut stun_request_message = Message::new(method, class);
    println!("stun_request_message: {}", stun_request_message);
    let message = stun_request_message.build();
    let response = stun_request(message, REMOTE_ADDRESS).await;
    // 受け取ったbytes数の内、20bytesはheaderなので、残りを読めば良い。
    println!("{:?}", &response);
    println!("{:?}", &response[20..]);
    let mut xor_addr = xor_addr::XorMappedAddress::new(
        response[20..].to_vec(),
        stun_request_message.transaction_id,
    );
    // xor_addr.get_from(&msg)?;
    // println!("{}", xor_addr);
    return "127.0.0.1".to_string();
}

#[tokio::main]
async fn main() {
    let my_global_ip: String = get_global_ip().await;
    // println!("{}", my_global_ip);
}
