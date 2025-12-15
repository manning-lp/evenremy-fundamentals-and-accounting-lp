use crate::AccountingError::AccountUnderFunded;
use std::collections::HashMap;
use std::io::{stdin, Write};

/// An application-specific error type
#[derive(Debug)]
enum AccountingError {
    AccountNotFound(String),
    AccountUnderFunded(String, u64),
    AccountOverFunded(String, u64),
}

/// A transaction type. Transactions should be able to rebuild a ledger's state
/// when they are applied in the same sequence to an empty state.
#[derive(Debug)]
pub enum Tx {
    Deposit { account: String, amount: u64 },
    Withdraw { account: String, amount: u64 },
}

/// A type for managing accounts and their current currency balance
#[derive(Debug)]
struct Accounts {
    accounts: HashMap<String, u64>, // id to amount
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
    pub fn deposit(&mut self, signer: &str, amount: u64) -> Result<Tx, AccountingError> {
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
                .map(|_| Tx::Deposit {
                    account: signer.to_string(),
                    amount,
                })
        } else {
            self.accounts.insert(signer.to_string(), amount);
            Ok(Tx::Deposit {
                account: signer.to_string(),
                amount,
            })
        }
    }

    /// Withdraws the `amount` from the `signer` account.
    /// # Errors
    /// Attempted overflow
    pub fn withdraw(&mut self, signer: &str, amount: u64) -> Result<Tx, AccountingError> {
        if let Some(account) = self.accounts.get_mut(signer) {
            account
                .checked_sub(amount)
                .and_then(|r| {
                    *account = r;
                    Some(r)
                })
                .ok_or(AccountUnderFunded(signer.to_string(), amount))
                .map(|_| Tx::Withdraw {
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
    ) -> Result<(Tx, Tx), AccountingError> {
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

fn read_from_stdin(label: &str) -> String {
    print!("{}", label);
    std::io::stdout().flush().unwrap_or_default();
    let mut line = String::new();
    stdin().read_line(&mut line).unwrap_or_else(|e| {
        line = "".to_string();
        return 0
    });

    line.trim().to_string()
}

fn main() {
    loop {
        let cmd = read_from_stdin("cmd:");
        println!("{}",cmd)
    }

    // println!("Hello, accounting world!");
    //
    // // We are using simple &str instances as keys
    // // for more sophisticated keys (e.g. hashes)
    // // the data type could remain the same
    // let bob = "bob";
    // let alice = "alice";
    // let charlie = "charlie";
    // let initial_amount = 100;
    //
    // // Creates the basic ledger and a tx log container
    // let mut ledger = Accounts::new();
    // let mut tx_log = vec![];
    //
    // // Deposit an amount to each account
    // for signer in [bob, alice, charlie] {
    //     let status = ledger.deposit(signer, initial_amount);
    //     println!("Depositing {} for {}: {:?}", signer, initial_amount, status);
    //     // Add the resulting transaction to a list of transactions
    //     // .unwrap() will crash the program if the status is an error.
    //     tx_log.push(status.unwrap());
    // }
    //
    // // Send currency from one account (bob) to the other (alice)
    // let send_amount = 10_u64;
    // let status = ledger.send(bob, alice, send_amount);
    // println!(
    //     "Sent {} from {} to {}: {:?}",
    //     send_amount, bob, alice, status
    // );
    //
    // // Add both transactions to the transaction log
    // let (tx1, tx2) = status.unwrap();
    // tx_log.push(tx1);
    // tx_log.push(tx2);
    //
    // // Withdraw everything from the accounts
    // let tx = ledger.withdraw(charlie, initial_amount).unwrap();
    // tx_log.push(tx);
    // let tx = ledger
    //     .withdraw(alice, initial_amount + send_amount)
    //     .unwrap();
    // tx_log.push(tx);
    //
    // // Here we are withdrawing too much and there won't be a transaction
    // println!(
    //     "Withdrawing {} from {}: {:?}",
    //     initial_amount,
    //     bob,
    //     ledger.withdraw(bob, initial_amount)
    // );
    // // Withdrawing the expected amount results in a transaction
    // let tx = ledger.withdraw(bob, initial_amount - send_amount).unwrap();
    // tx_log.push(tx);
    //
    // // {:?} prints the Debug implementation, {:#?} pretty-prints it
    // println!("Ledger empty: {:?}", ledger);
    // println!("The TX log: {:#?}", tx_log);
    //
    // // add max u64 to alice and bob
    // let tx = ledger.deposit(alice, u64::MAX).unwrap();
    // tx_log.push(tx);
    // let tx = ledger.deposit(bob, u64::MAX).unwrap();
    // tx_log.push(tx);
    // println!("Ledger maxed: {:?}", ledger);
    // let expected_amount = *ledger.accounts.get(alice).unwrap();
    // let e = ledger.send(alice, bob, 1000).unwrap_err();
    // println!("send error: {:?}", e);
    // println!(
    //     "alice refunded: {}, bob untouched: {}",
    //     *ledger.accounts.get(alice).unwrap() == expected_amount,
    //     *ledger.accounts.get(bob).unwrap() == expected_amount
    // );
}
