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

#[derive(Debug, Copy, Clone)]
pub struct AttributeType(pub u16);

pub const ATTR_XORMAPPED_ADDRESS: AttributeType = AttributeType(0x0020);
pub const ATTR_UNKNOWN_ATTRIBUTES: AttributeType = AttributeType(0x000A);