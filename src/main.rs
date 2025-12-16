mod accounts;
mod errors;
mod tx;

use std::io::{stdin, Write};

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
    let mut ledger = accounts::Accounts::new();
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

fn cmd_send(
    ledger: &mut accounts::Accounts,
    tx_log: &mut Vec<tx::Tx>,
    amount: &&str,
    from: &&str,
    to: &&str,
) {
    if let Ok(amount) = amount.parse::<u64>() {
        match ledger.send(from, to, amount) {
            Ok((Tx1, Tx2)) => tx_log.append(vec![Tx1, Tx2].as_mut()),
            Err(e) => {
                eprintln!("{:?}", e)
            }
        }
    } else {
        eprintln!("failed to parse '{}'", amount);
    };
}

fn cmd_deposit(
    ledger: &mut accounts::Accounts,
    tx_log: &mut Vec<tx::Tx>,
    amount: &&str,
    signer: &&str,
) {
    if let Ok(amount) = amount.parse::<u64>() {
        match ledger.deposit(signer, amount) {
            Ok(Tx) => tx_log.push(Tx),
            Err(e) => {
                eprintln!("{:?}", e)
            }
        }
    } else {
        eprintln!("failed to parse '{}'", amount);
    };
}

fn cmd_withdraw(
    ledger: &mut accounts::Accounts,
    tx_log: &mut Vec<tx::Tx>,
    amount: &&str, // todo get rid of one ref
    signer: &&str,
) {
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
