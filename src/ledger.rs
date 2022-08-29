use crate::{account::Account, error::Error, ClientId, Transaction, TxId};
use std::collections::{HashMap, HashSet};
use tracing::debug;

pub struct Ledger {
    accounts: HashMap<ClientId, Account>,
    transactions: HashSet<TxId>,
}

impl Default for Ledger {
    fn default() -> Self {
        Self::new()
    }
}

impl Ledger {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            transactions: HashSet::new(),
        }
    }

    /// Public interface for processing transactions
    ///
    /// # Example
    /// ```rust
    /// use ledger_rs::{Ledger, Transaction};
    ///
    /// let mut ledger = Ledger::new();
    /// let tx = Transaction::Dispute { id: 1, client: 1 };
    ///
    /// assert!(ledger.process_tx(tx).is_err());
    /// ```
    /// # Errors
    /// Could return an error, see [`Error`] for more
    pub fn process_tx(&mut self, tx: Transaction) -> Result<(), Error> {
        let id = tx.id();
        let client = tx.client();
        let account = self.accounts.entry(client).or_insert_with(|| {
            debug!("Account with ID {client} not found, creating new account");
            Account::new(client)
        });

        if let Transaction::Deposit { .. } | Transaction::Withdrawal { .. } = tx {
            if self.transactions.contains(&id) {
                return Err(Error::DuplicateTxId(id));
            }
            self.transactions.insert(id);
            account.process_tx(tx)
        } else {
            account.process_tx(tx)
        }
    }

    /// Gets all accounts
    ///
    /// # Example
    ///
    /// ```rust
    /// use ledger_rs::{Transaction, Ledger};
    /// use rust_decimal::Decimal;
    ///
    /// let mut ledger = Ledger::new();
    ///
    /// assert!(ledger
    ///     .process_tx(Transaction::Deposit {
    ///         id: 1,
    ///         client: 1,
    ///         amount: Decimal::new(2, 0),
    ///     })
    ///     .is_ok());
    /// assert_eq!(ledger.get_accounts().count(), 1);
    /// ```
    pub fn get_accounts(&self) -> impl Iterator<Item = &Account> {
        self.accounts.values()
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;

    use crate::{error::Error, Ledger, Transaction};

    #[test]
    fn duplicate_tx_id() {
        // Setup
        let id = 1;
        let client = 1;
        let amount = Decimal::new(2, 0);
        let tx = Transaction::Deposit { id, client, amount };
        let mut ledger = Ledger::default();

        assert!(ledger
            .process_tx(Transaction::Deposit { id, client, amount })
            .is_ok());

        // Act
        let result = ledger.process_tx(tx);

        // Assert
        assert!(matches!(result, Err(Error::DuplicateTxId(..))));
    }
}
