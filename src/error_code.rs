// RFC8489 STUN section 14.8 ErrorCode
// 300~699のエラーコードで表す。
// 先頭20bitは空白, Class = 4bit, Number = 8bit
// Classはエラーコードの先頭1文字を表し、3~6で表現する(なので4bit必要)
// Numberはエラーコードの末尾2文字を表し、0~99で表現する(なので、残りの8bitを使う)
// 0                   1                   2                   3
//0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
//+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//|           Reserved, should be 0         |Class|     Number    |
//+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//|      Reason Phrase (variable)                                ..
//+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

use crate::attribute::{Attribute, ATTR_ERROR_CODE};
use crate::error::*;
use crate::message::{Message, Setter};
pub struct ErrorCode(pub u16);

pub const CODE_TRY_ALTERNATE: ErrorCode = ErrorCode(300);
pub const CODE_BAD_REQUEST: ErrorCode = ErrorCode(400);
pub const CODE_UNAUTHORIZED: ErrorCode = ErrorCode(401);
pub const CODE_UNKNOWN_ATTRIBUTE: ErrorCode = ErrorCode(420);
pub const CODE_STALE_NONCE: ErrorCode = ErrorCode(438);
pub const CODE_SERVER_ERROR: ErrorCode = ErrorCode(500);

pub struct ErrorCodeAttribute {
    pub code: ErrorCode,
    pub reason: Vec<u8>,
}

const ERROR_CODE_MODULO: u16 = 100; // 100の位でMODULO = 徐算
const MAX_REASON_PHRASE_BYTES: usize = 763;

impl Setter for ErrorCodeAttribute {
    fn set_extra_attribute(&self, m: &mut Message) -> Result<()> {
        let mut raw = Vec::with_capacity(MAX_REASON_PHRASE_BYTES);
        // 先頭20bitは空白, Class = 4bit, Number = 8bit
        raw.extend_from_slice(&[0, 0]); // 2bytes分は0で埋める
        let class = (self.code.0 / ERROR_CODE_MODULO) as u8; // Classはエラーコードの先頭1文字を表し、3~6で表現する(なので4bit必要)
        let number: u8 = (self.code.0 % ERROR_CODE_MODULO) as u8; // Numberはエラーコードの末尾2文字を表し、0~99で表現する(なので、残りの8bitを使う)
        raw.push(class);
        raw.push(number);
        raw.extend_from_slice(&self.reason);
        let extra_attribute = Attribute::new(ATTR_ERROR_CODE, MAX_REASON_PHRASE_BYTES as u16, raw);
        m.attributes.push(extra_attribute);
        Ok(())
    }
}
