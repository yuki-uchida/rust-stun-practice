use std::fmt;
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
