pub struct ErrorCode(pub u16);

pub const CODE_TRY_ALTERNATE: ErrorCode = ErrorCode(300);
pub const CODE_BAD_REQUEST: ErrorCode = ErrorCode(400);
pub const CODE_UNAUTHORIZED: ErrorCode = ErrorCode(401);
pub const CODE_UNKNOWN_ATTRIBUTE: ErrorCode = ErrorCode(420);
pub const CODE_STALE_NONCE: ErrorCode = ErrorCode(438);
pub const CODE_SERVER_ERROR: ErrorCode = ErrorCode(500);
