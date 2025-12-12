#[repr(i16)]
#[derive(Clone)]
pub enum ZkErrorCode {
    InvalidAttestationLength = 1001,
    ParseConfigData,
    VerifyAttestation,
    InvalidRequestLength,
    InvalidMessagesLength,
    GetJsonValueFail,
    InvalidJsonValueSize,
    CannotFoundTimestamp,
    ParseTimestampFailed,
    InvalidRequestOrder,
    InvalidRequestUrl,
    DuplicateAccount,
}

pub struct ZktlsError {
    code: ZkErrorCode,
    msg: String,
}

impl ZktlsError {
    pub fn new(code: ZkErrorCode, msg: impl Into<String>) -> Self {
        Self { code, msg: msg.into() }
    }
    pub fn icode(&self) -> i16 {
        self.code.clone() as i16
    }
    pub fn msg(&self) -> String {
        self.msg.clone()
    }
}

#[macro_export]
macro_rules! ensure_zk {
    ($cond:expr, $err:expr) => {
        if !$cond {
            return Err($err.into());
        }
    };
}

#[macro_export]
macro_rules! zkerr {
    ($code:expr, $msg:expr) => {
        ZktlsError::new($code, $msg)
    };
    ($code:expr) => {
        zkerr!($code, stringify!($code).to_string())
    };
}
