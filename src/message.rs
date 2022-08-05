use crate::attribute::*;
use crate::error::*;
use rand::Rng;
use std::convert::TryInto;
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
pub(crate) const MAGIC_COOKIE: u32 = 0x2112A442; // 32bit = 4bytes
const ATTRIBUTE_HEADER_SIZE: usize = 4;
const MESSAGE_HEADER_SIZE: usize = 20; // 160bit = 20bytes
const TRANSACTION_ID_SIZE: usize = 12; // 96bit = 12 bytes
const DEFAULT_RAW_CAPACITY: usize = 120; //960bit = 120bytes

#[derive(Debug, Default)]
pub struct Message {
    pub method: Method,
    pub class: MethodClass,
    pub attributes: Vec<Attribute>,
    pub transaction_id: [u8; TRANSACTION_ID_SIZE],
}

pub trait Setter {
    // STUNの拡張であるTURN側からもmessageに拡張のattributeを追加するためにSetterを追加
    fn set_extra_attribute(&self, m: &mut Message) -> Result<()>;
}
impl Message {
    pub fn new(method: Method, class: MethodClass) -> Self {
        let mut random_transaction_id = [0u8; TRANSACTION_ID_SIZE];
        rand::thread_rng().fill(&mut random_transaction_id);
        Message {
            method: method,
            class: class,
            attributes: Vec::new(),
            transaction_id: random_transaction_id,
        }
    }
    pub fn contains(&self, t: AttributeType) -> bool {
        for a in &self.attributes {
            if a.typ == t {
                return true;
            }
        }
        return false;
    }
    // This function calls extra attribute's Setter(set_extra_attribute).
    // dyn = トレイトオブジェクトであることを明示する.ここでは型ではなくトレイトを引数に渡している。
    // 具体的な型は隠蔽し、トレイトオブジェクトであることだけを確認する。これによって、setterに値するものがどんな型であっても問題ない。
    // TURNのRequestedTransportなどがこれを利用する。
    pub fn set_extra_attribute(&mut self, setter: Box<dyn Setter>) -> Result<()> {
        setter.set_extra_attribute(self)?;
        Ok(())
    }

    pub fn set_xor_mapped_address(&mut self, remote_ip: SocketAddr) {
        let (ip, port) = (remote_ip.ip(), remote_ip.port() as u32);
        let (mut raw, length) = if remote_ip.is_ipv4() {
            (Vec::with_capacity(8), 8) // 8bytes=64bits
        } else {
            (Vec::with_capacity(20), 20) // 20bytes=160bits
        };

        match ip {
            IpAddr::V4(ipv4) => {
                // 8bytes=64bits
                raw.extend_from_slice(&[0; 8]);
                // family
                let family: u8 = 0x01;
                raw[1..2].copy_from_slice(&family.to_be_bytes());
                // port
                let port = ((port as u16) ^ (MAGIC_COOKIE >> 16) as u16) as u16;
                raw[2..4].copy_from_slice(&port.to_be_bytes());
                // address
                let ip_addr: u32 = u32::from_be_bytes(ipv4.octets().try_into().unwrap());
                let xor_ip_addr = (ip_addr as u32) ^ MAGIC_COOKIE;
                raw[4..8].copy_from_slice(&xor_ip_addr.to_be_bytes());
                println!("raw: {:?}", raw);
            }
            IpAddr::V6(ipv6) => { /* handle IPv6 */ }
        }
        let xor_mapped_address_attribute = Attribute::new(ATTR_XORMAPPED_ADDRESS, length, raw);
        self.attributes.push(xor_mapped_address_attribute);
    }
    pub fn encode_to_packet(&mut self) -> Vec<u8> {
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
        let mut index = MESSAGE_HEADER_SIZE;
        // Attributes
        for attribute in self.attributes.iter() {
            // Attributeのサイズ分あらかじめ配列を長くしておく
            if attribute.length % 4 == 0 {
                let attribute_length: usize =
                    (ATTRIBUTE_HEADER_SIZE as u16 + attribute.length) as usize;
                raw.extend_from_slice(&vec![0; attribute_length]);
            } else {
                let attribute_length: usize =
                    (ATTRIBUTE_HEADER_SIZE as u16 + (((attribute.length / 4) + 1) * 4)) as usize;
                raw.extend_from_slice(&vec![0; attribute_length]);
            }
            // Type
            raw[index..index + 2].copy_from_slice(&attribute.typ.0.to_be_bytes());
            index += 2;

            // Length
            raw[index..index + 2].copy_from_slice(&attribute.length.to_be_bytes());
            index += 2;

            // Value
            println!("raw={:?}", raw);
            println!("index={:?} attribute.length={:?}", index, attribute.length);
            println!("index={:?} attribute.value={:?}", index, attribute.value);
            // サイズが合ってない。lengthが763bytesなのに、実際には数十しかないので、`panicked at 'source slice length (16) does not match destination slice length (763)` がでる
            raw[index..(index + attribute.length as usize)].copy_from_slice(&attribute.value);
            if attribute.length % 4 == 0 {
                index += attribute.length as usize
            } else {
                index += (((attribute.length / 4) + 1) * 4) as usize;
            }
            println!(
                "index={:?}, attribute.typ={:?}, attribute.length={:?}, attribute.value={:?} raw={:?}",
                index, attribute.typ, attribute.length, attribute.value, raw
            );
        }
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
        const CLASS_LEFT_BIT: u16 = 0x2; // 0b10
        const CLASS_RIGHT_BIT: u16 = 0x1; // 0b01
        const CLASS_LEFT_SHIFT: u16 = 7;
        const CLASS_RIGHT_SHIFT: u16 = 4;

        // method
        let method = self.method.0 as u16;
        let right_bit = method & RIGHT_BIT;
        let centor_bit = (method & CENTOR_BIT) << METHOD_CENTOR_SHIFT;
        let left_bit = (method & LEFT_BIT) << METHOD_LEFT_DSHIFT;
        let method = left_bit + centor_bit + right_bit;

        // class
        let class = self.class.0 as u16;
        let c1 = (class & CLASS_LEFT_BIT) << CLASS_LEFT_SHIFT;
        let c0 = (class & CLASS_RIGHT_BIT) << CLASS_RIGHT_SHIFT;
        println!(
            "class {:?}, c1(class_left_bit): {:?}, c0(class_right_bit): {:?}",
            class, c1, c0
        );
        let class = c1 + c0;
        let message_type_bytes = method + class;
        return message_type_bytes;
    }
    fn build_message_length(&mut self) -> u16 {
        //TODO: impl attributes and message_length
        let mut stun_message_length = 0 as u16;
        for attribute in self.attributes.iter() {
            stun_message_length += ATTRIBUTE_HEADER_SIZE as u16;
            if attribute.length % 4 == 0 {
                stun_message_length += attribute.length as u16;
            } else {
                stun_message_length += (((attribute.length / 4) + 1) * 4) as u16;
            }
        }
        return stun_message_length;
    }

    pub fn decode_from_packet(packet: &Vec<u8>) -> Result<Self> {
        // request method type
        const RIGHT_BIT: u16 = 0xf; // 0b0000000000001111
        const CENTOR_BIT: u16 = 0x70; // 0b0000000001110000
        const LEFT_BIT: u16 = 0xf80; // 0b0000111110000000
        const METHOD_CENTOR_SHIFT: u16 = 1;
        const METHOD_LEFT_DSHIFT: u16 = 2;
        let right_bit = u16::from_be_bytes([packet[0], packet[1]]) & RIGHT_BIT;
        let centor_bit =
            (u16::from_be_bytes([packet[0], packet[1]]) & CENTOR_BIT) << METHOD_CENTOR_SHIFT;
        let left_bit =
            (u16::from_be_bytes([packet[0], packet[1]]) & LEFT_BIT) << METHOD_LEFT_DSHIFT;
        let method_bit = left_bit + centor_bit + right_bit;
        let method = Method(method_bit);
        // request class type
        const CLASS_LEFT_BIT: u16 = 0x100; // 0b0000000100000000
        const CLASS_RIGHT_BIT: u16 = 0x010; // 0b0000000000010000
        let c1 = u16::from_be_bytes([packet[0], packet[1]]) & CLASS_LEFT_BIT;
        let c0 = u16::from_be_bytes([packet[0], packet[1]]) & CLASS_RIGHT_BIT;
        let class_bit = c1 as u8 + c0 as u8;
        let class = MethodClass(class_bit);
        // cookie
        if u32::from_be_bytes([packet[4], packet[5], packet[6], packet[7]]) != MAGIC_COOKIE {
            return Err(Error::new(format!(
                "{:x} is invalid magic cookie (should be {:x})",
                u32::from_be_bytes([packet[4], packet[5], packet[6], packet[7]]),
                MAGIC_COOKIE
            ))
            .into());
        }

        // attributes
        let mut attributes = Vec::new();
        // type length value
        // HEADER SIZE 4
        // 1個で8bit
        // typeは16bit(2byte) lengthは16bit(2byte)
        // valueは謎だが32bit単位(4byte)
        let mut attribute_start_index: usize = 20;
        loop {
            if packet.len() == attribute_start_index {
                break;
            }
            let t = u16::from_be_bytes([
                packet[attribute_start_index],
                packet[attribute_start_index + 1],
            ]);
            let l = u16::from_be_bytes([
                packet[attribute_start_index + 2],
                packet[attribute_start_index + 3],
            ]);
            let v = &packet[(attribute_start_index + 4)..(attribute_start_index + 4 + l as usize)];
            let attribute: Attribute = match t {
                0x0019 => Attribute {
                    typ: ATTR_REQUESTED_TRANSPORT,
                    length: l,
                    value: v.to_vec(),
                },
                0x0020 => Attribute {
                    typ: ATTR_XORMAPPED_ADDRESS,
                    length: l,
                    value: v.to_vec(),
                },
                0x0014 => Attribute {
                    typ: ATTR_REALM,
                    length: l,
                    value: v.to_vec(),
                },
                0x0015 => Attribute {
                    typ: ATTR_NONCE,
                    length: l,
                    value: v.to_vec(),
                },
                0x0006 => Attribute {
                    typ: ATTR_USERNAME,
                    length: l,
                    value: v.to_vec(),
                },
                0x0008 => Attribute {
                    typ: ATTR_MESSAGE_INTEGRITY,
                    length: l,
                    value: v.to_vec(),
                },
                0x0009 => Attribute {
                    typ: ATTR_ERROR_CODE,
                    length: l,
                    value: v.to_vec(),
                },
                0x8028 => Attribute {
                    typ: ATTR_FINGERPRINT,
                    length: l,
                    value: v.to_vec(),
                },
                _ => Attribute {
                    typ: ATTR_UNKNOWN_ATTRIBUTES,
                    length: l,
                    value: v.to_vec(),
                },
            };
            if l % 4 == 0 {
                attribute_start_index += 4 + l as usize;
            } else {
                attribute_start_index += 4 + (((l / 4) + 1) * 4) as usize;
            }
            attributes.push(attribute);
        }
        Ok(Message {
            method: method,
            class: class,
            attributes: attributes,
            transaction_id: packet[8..20].try_into().unwrap(), // to transform slice into array.
        })
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

#[derive(PartialEq, Eq, Debug, Default)]
pub struct Method(u16);
pub const METHOD_BINDING: Method = Method(0x001);
pub const METHOD_ALLOCATE: Method = Method(0x003);
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

#[derive(PartialEq, Eq, Debug, Default)]
pub struct MethodClass(u8);
pub const CLASS_REQUEST: MethodClass = MethodClass(0x00); // 0b00: request
pub const CLASS_INDICATION: MethodClass = MethodClass(0x01); // 0b01: indication
pub const CLASS_SUCCESS: MethodClass = MethodClass(0x02); // 0b10: success
pub const CLASS_ERROR: MethodClass = MethodClass(0x03); // 0b11: error
impl fmt::Display for MethodClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match *self {
            CLASS_REQUEST => "CLASS_REQUEST",
            CLASS_INDICATION => "CLASS_INDICATION",
            CLASS_SUCCESS => "CLASS_SUCCESS",
            CLASS_ERROR => "CLASS_ERROR",
            _ => "unknown class",
        };
        write!(f, "{}", s)
    }
}
