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
        return 0;
    });

    line.trim().to_string()
}

fn main() {
    let mut ledger = Accounts::new();
    let mut tx_log = vec![];
    loop {
        let line = read_from_stdin("cmd: ");
        let cmd: Vec<&str> = line.split(" ").collect();
        match cmd.as_slice() {
            ["deposit", amount, "to", signer] => {
                cmd_deposit(&mut ledger, &mut tx_log, amount, signer);
            }
            ["withdraw", amount, "from", signer] => {
                cmd_withdraw(&mut ledger, &mut tx_log, amount, signer);
            }
            ["send", amount, "from", from, "to", to] => {
                cmd_send(&mut ledger, &mut tx_log, amount, from, to);
            }
            ["print"] => {
                println!("{:?}", ledger)
            }
            ["quit"] => {
                return;
            }
            _ => println!(
                "Command '{}' not found",
                cmd.first().unwrap_or_else(|| { &"" })
            ),
        }
    }
}

fn cmd_send(ledger: &mut Accounts, tx_log: &mut Vec<Tx>, amount: &&str, from: &&str, to: &&str) {
    if let Ok(amount) = amount.parse::<u64>() {
        match ledger.send(from, to, amount) {
            Ok((tx1, tx2)) => tx_log.append(vec![tx1, tx2].as_mut()),
            Err(e) => {
                eprintln!("{:?}", e)
            }
        }
    } else {
        eprintln!("failed to parse '{}'", amount);
    };
}

fn cmd_deposit(ledger: &mut Accounts, tx_log: &mut Vec<Tx>, amount: &&str, signer: &&str) {
    if let Ok(amount) = amount.parse::<u64>() {
        match ledger.deposit(signer, amount) {
            Ok(tx) => tx_log.push(tx),
            Err(e) => {
                eprintln!("{:?}", e)
            }
        }
    } else {
        eprintln!("failed to parse '{}'", amount);
    };
}

fn cmd_withdraw(ledger: &mut Accounts, tx_log: &mut Vec<Tx>, amount: &&str, signer: &&str) {
    if let Ok(amount) = amount.parse::<u64>() {
        match ledger.withdraw(signer, amount) {
            Ok(tx) => tx_log.push(tx),
            Err(e) => {
                eprintln!("{:?}", e)
            }
        }
    } else {
        eprintln!("failed to parse '{}'", amount);
    };
}
