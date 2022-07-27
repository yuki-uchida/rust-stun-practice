use crate::error::*;
use crate::message::{Message, Setter};
use std::fmt;

const MAX_NONCE_BYTE: usize = 763;
const MAX_REALM_BYTE: usize = 763;

#[derive(Debug, Clone)]
pub struct Attribute {
    pub typ: AttributeType,
    pub length: u16,
    pub value: Vec<u8>,
}
impl Attribute {
    pub fn new(typ: AttributeType, length: u16, value: Vec<u8>) -> Self {
        Attribute {
            typ: typ,
            length: length,
            value: value,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextAttribute {
    pub attr: AttributeType,
    pub text: String,
}
impl TextAttribute {
    pub fn new(attr: AttributeType, text: String) -> Self {
        TextAttribute { attr, text }
    }
}
impl fmt::Display for TextAttribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl Setter for TextAttribute {
    fn set_extra_attribute(&self, m: &mut Message) -> Result<()> {
        let text = self.text.as_bytes();
        let max_len = match self.attr {
            ATTR_NONCE => MAX_NONCE_BYTE,
            ATTR_REALM => MAX_REALM_BYTE,
            _ => return Err(Error::Other(format!("Unsupported AttributeType"))),
        };
        // let (mut raw, length) = (
        //     Vec::with_capacity(REQUESTED_TRANSPORT_SIZE),
        //     REQUESTED_TRANSPORT_SIZE,
        // );
        // // extra_attribute
        // raw.extend_from_slice(&[0; REQUESTED_TRANSPORT_SIZE]);
        // raw[0] = self.protocol.0;
        let extra_attribute = Attribute::new(ATTR_REQUESTED_TRANSPORT, MAX_NONCE_B as u16, raw);
        m.attributes.push(extra_attribute);
        Ok(())
    }
}

pub type Nonce = TextAttribute;
pub type Realm = TextAttribute;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct AttributeType(pub u16);
impl fmt::Display for AttributeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match *self {
            ATTR_REQUESTED_TRANSPORT => "ATTR_REQUESTED_TRANSPORT",
            ATTR_XORMAPPED_ADDRESS => "ATTR_XORMAPPED_ADDRESS",
            ATTR_UNKNOWN_ATTRIBUTES => "ATTR_UNKNOWN_ATTRIBUTES",
            ATTR_REALM => "ATTR_REALM",
            ATTR_NONCE => "ATTR_NONCE",
            ATTR_USERNAME => "ATTR_USERNAME",
            ATTR_MESSAGE_INTEGRITY => "ATTR_MESSAGE_INTEGRITY",
            ATTR_FINGERPRINT => "ATTR_FINGERPRINT",
            _ => "unknown AttributeType",
        };
        write!(f, "{}", s)
    }
}

// for stun
pub const ATTR_REQUESTED_TRANSPORT: AttributeType = AttributeType(0x0019);
pub const ATTR_XORMAPPED_ADDRESS: AttributeType = AttributeType(0x0020);
pub const ATTR_UNKNOWN_ATTRIBUTES: AttributeType = AttributeType(0x000A);
pub const ATTR_ERROR_CODE: AttributeType = AttributeType(0x0009); // ERROR-CODE
                                                                  // for turn autihorize
pub const ATTR_REALM: AttributeType = AttributeType(0x0014);
pub const ATTR_NONCE: AttributeType = AttributeType(0x0015);
pub const ATTR_USERNAME: AttributeType = AttributeType(0x0006);
pub const ATTR_MESSAGE_INTEGRITY: AttributeType = AttributeType(0x0008);
pub const ATTR_FINGERPRINT: AttributeType = AttributeType(0x8028);
