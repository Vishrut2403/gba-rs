#[derive(Debug, PartialEq, Eq)]
pub enum DecodeError {
    InvalidCondition(u8),
}