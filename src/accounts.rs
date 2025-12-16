pub mod accounts {
    use crate::{error, tx};
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
        pub fn deposit(
            &mut self,
            signer: &str,
            amount: u64,
        ) -> Result<tx::Tx, error::AccountingError> {
            if let Some(account) = self.accounts.get_mut(signer) {
                (*account)
                    .checked_add(amount)
                    .and_then(|r| {
                        *account = r;
                        Some(r)
                    })
                    .ok_or(error::AccountingError::AccountOverFunded(
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
        pub fn withdraw(
            &mut self,
            signer: &str,
            amount: u64,
        ) -> Result<tx::Tx, error::AccountingError> {
            if let Some(account) = self.accounts.get_mut(signer) {
                account
                    .checked_sub(amount)
                    .and_then(|r| {
                        *account = r;
                        Some(r)
                    })
                    .ok_or(error::AccountingError::AccountUnderFunded(
                        signer.to_string(),
                        amount,
                    ))
                    .map(|_| tx::Tx::Withdraw {
                        account: signer.to_string(),
                        amount,
                    })
            } else {
                Err(error::AccountingError::AccountNotFound(signer.to_string()))
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
        ) -> Result<(tx::Tx, tx::Tx), error::AccountingError> {
            let Some(_) = self.accounts.get_mut(sender) else {
                return Err(error::AccountingError::AccountNotFound(sender.to_string()));
            };
            let Some(_) = self.accounts.get_mut(recipient) else {
                return Err(error::AccountingError::AccountNotFound(
                    recipient.to_string(),
                ));
            };
            let Ok(withdraw) = self.withdraw(sender, amount) else {
                return Err(error::AccountingError::AccountUnderFunded(
                    sender.to_string(),
                    amount,
                ));
            };
            let Ok(deposit) = self.deposit(recipient, amount) else {
                // return the amount to sender
                self.deposit(sender, amount)?;
                return Err(error::AccountingError::AccountOverFunded(
                    recipient.to_string(),
                    amount,
                ));
            };
            Ok((withdraw, deposit))
        }
    }
}