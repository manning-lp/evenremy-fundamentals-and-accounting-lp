/// An application-specific error type
#[derive(Debug)]
pub(crate) enum AccountingError {
    AccountNotFound(String),
    AccountUnderFunded(String, u64),
    AccountOverFunded(String, u64),
}
