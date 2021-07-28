use std::net::{IpAddr, Ipv4Addr};
pub(crate) struct XorMappedAddress {
    pub family: u8, // enumにしたい
    pub ip: IpAddr,
    pub port: u16,
}

const MAGIC_COOKIE: u32 = 0x2112A442; // 32bit = 4bytes
const TRANSACTION_ID_SIZE: usize = 12;
const IPV4LEN: usize = 4;
fn safe_xorbytes(dst: &mut [u8], a: &[u8], b: &[u8]) {
    let mut n = a.len();
    if b.len() < n {
        n = b.len();
    }
    if dst.len() < n {
        n = dst.len();
    }
    for i in 0..n {
        dst[i] = a[i] ^ b[i];
    }
}

impl XorMappedAddress {
    pub fn new(attributes_payload: Vec<u8>, transaction_id: [u8; TRANSACTION_ID_SIZE]) -> Self {
        let family = attributes_payload[5..6][0];
        let port = u16::from_be_bytes([attributes_payload[6], attributes_payload[7]])
            ^ (MAGIC_COOKIE >> 16) as u16;

        let mut xor_value = vec![0; 4 + TRANSACTION_ID_SIZE];
        xor_value[0..4].copy_from_slice(&MAGIC_COOKIE.to_be_bytes());
        xor_value[4..].copy_from_slice(&transaction_id);

        //TODO: impl for IPv6
        let mut ip = [0; IPV4LEN];
        safe_xorbytes(&mut ip, &attributes_payload[8..], &xor_value);
        let global_ip = IpAddr::V4(Ipv4Addr::from(ip));
        XorMappedAddress {
            family: family,
            ip: global_ip,
            port: port,
        }
    }
}
