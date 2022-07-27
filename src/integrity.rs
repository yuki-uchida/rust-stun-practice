// RFC 5389 Section 15.4
pub struct MessageIntegrity(pub Vec<u8>);
pub const MESSAGE_INTEGRITY_SIZE: usize = 20;

impl MessageIntegrity {}
