use crate::errors::AccountingError;
use crate::tx;
use std::collections::HashMap;

/// A type for managing accounts and their current currency balance
#[derive(Debug)]
pub struct Accounts {
    pub accounts: HashMap<String, u64>, // id to amount
}

impl Accounts {
    /// Returns an empty instance of the [`Accounts`] type
    pub fn new() -> Self {
        Accounts {
            accounts: Default::default(),
        }
    }

    /// Either deposits the `amount` provided into the `signer` account or adds the amount to the existing account.
    /// # Errors
    /// Attempted overflow
    pub fn deposit(&mut self, signer: &str, amount: u64) -> Result<tx::Tx, AccountingError> {
        if let Some(account) = self.accounts.get_mut(signer) {
            (*account)
                .checked_add(amount)
                .and_then(|r| {
                    *account = r;
                    Some(r)
                })
                .ok_or(AccountingError::AccountOverFunded(
                    signer.to_string(),
                    amount,
                ))
                // Using map() here is an easy way to only manipulate the non-error result
                .map(|_| tx::Tx::Deposit {
                    account: signer.to_string(),
                    amount,
                })
        } else {
            self.accounts.insert(signer.to_string(), amount);
            Ok(tx::Tx::Deposit {
                account: signer.to_string(),
                amount,
            })
        }
    }

    /// Withdraws the `amount` from the `signer` account.
    /// # Errors
    /// Attempted overflow
    pub fn withdraw(&mut self, signer: &str, amount: u64) -> Result<tx::Tx, AccountingError> {
        if let Some(account) = self.accounts.get_mut(signer) {
            account
                .checked_sub(amount)
                .and_then(|r| {
                    *account = r;
                    Some(r)
                })
                .ok_or(AccountingError::AccountUnderFunded(
                    signer.to_string(),
                    amount,
                ))
                .map(|_| tx::Tx::Withdraw {
                    account: signer.to_string(),
                    amount,
                })
        } else {
            Err(AccountingError::AccountNotFound(signer.to_string()))
        }
    }

    /// Withdraws the amount from the sender account and deposits it in the recipient account.
    ///
    /// # Errors
    /// The account doesn't exist
    pub fn send(
        &mut self,
        sender: &str,
        recipient: &str,
        amount: u64,
    ) -> Result<(tx::Tx, tx::Tx), AccountingError> {
        let Some(_) = self.accounts.get_mut(sender) else {
            return Err(AccountingError::AccountNotFound(sender.to_string()));
        };
        let Some(_) = self.accounts.get_mut(recipient) else {
            return Err(AccountingError::AccountNotFound(recipient.to_string()));
        };
        let Ok(withdraw) = self.withdraw(sender, amount) else {
            return Err(AccountingError::AccountUnderFunded(
                sender.to_string(),
                amount,
            ));
        };
        let Ok(deposit) = self.deposit(recipient, amount) else {
            // return the amount to sender
            self.deposit(sender, amount)?;
            return Err(AccountingError::AccountOverFunded(
                recipient.to_string(),
                amount,
            ));
        };
        Ok((withdraw, deposit))
    }
}

#[cfg(test)]
mod tests {
    use crate::accounts::Accounts;
    use crate::errors::AccountingError;
    use crate::tx;

    #[test]
    fn test_accounts_withdraw_underfunded() {
        let mut accounts = Accounts::new();
        accounts.deposit("alice", 100).unwrap();
        let error = accounts.withdraw("alice", 200);
        let expected = Err(AccountingError::AccountUnderFunded(
            "alice".to_string(),
            200,
        ));
        assert_eq!(error, expected);
    }

    #[test]
    fn test_accounts_deposit_overfunded() {
        let mut accounts = Accounts::new();
        accounts.deposit("alice", 100).unwrap();
        let error = accounts.deposit("alice", u64::MAX);
        let expected = Err(AccountingError::AccountOverFunded(
            "alice".to_string(),
            u64::MAX,
        ));
        assert_eq!(error, expected);
    }

    #[test]
    fn test_accounts_not_found() {
        let mut accounts = Accounts::new();
        let error = accounts.withdraw("alice", u64::MAX);
        let expected = Err(AccountingError::AccountNotFound("alice".to_string()));
        assert_eq!(error, expected);
    }

    #[test]
    fn test_accounts_deposit_success() {
        let mut accounts = Accounts::new();
        accounts.deposit("alice", 100).unwrap();
        let tx = accounts.deposit("alice", 100);
        let expected = Ok(tx::Tx::Deposit {
            account: "alice".to_string(),
            amount: 100,
        });
        assert_eq!(tx, expected);
    }

    #[test]
    fn test_accounts_withdraw_success() {
        let mut accounts = Accounts::new();
        accounts.deposit("alice", u64::MAX).unwrap();
        let tx = accounts.withdraw("alice", u64::MAX);
        let expected = Ok(tx::Tx::Withdraw {
            account: "alice".to_string(),
            amount: u64::MAX,
        });
        assert_eq!(tx, expected);
    }
}
