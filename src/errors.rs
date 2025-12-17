/// An application-specific error type
#[derive(Debug, PartialEq)]
pub enum AccountError {
    NotFound(String),
    UnderFunded(String, u64),
    OverFunded(String, u64),
}
