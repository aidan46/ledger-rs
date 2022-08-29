use crate::{error::Error, transaction::TransactionState, ClientId, Transaction, TxId};
use rust_decimal::Decimal;
use serde::Serialize;
use std::collections::{hash_map::Entry, HashMap};

#[derive(Debug, Serialize)]
pub struct Account {
    #[serde(rename(serialize = "client"))]
    pub id: ClientId,
    available: Decimal,
    held: Decimal,
    total: Decimal,
    locked: bool,
    #[serde(skip)]
    transactions: HashMap<TxId, Transaction>,
    #[serde(skip)]
    tx_state: HashMap<TxId, TransactionState>,
}

impl Account {
    pub(crate) fn new(id: ClientId) -> Self {
        Self {
            id,
            available: Decimal::ZERO,
            held: Decimal::ZERO,
            total: Decimal::ZERO,
            locked: false,
            transactions: HashMap::new(),
            tx_state: HashMap::new(),
        }
    }

    pub(crate) fn process_tx(&mut self, tx: Transaction) -> Result<(), Error> {
        if tx.client() != self.id {
            return Err(Error::Unauthorized(tx.client(), self.id));
        }
        if self.locked {
            return Err(Error::Locked(self.id));
        }
        match tx {
            Transaction::Deposit { id, amount, .. } => match self.deposit(id, amount) {
                Ok(()) => {
                    self.transactions.insert(id, tx);
                    Ok(())
                }
                Err(e) => Err(e),
            },
            Transaction::Withdrawal { id, amount, .. } => match self.withdrawal(amount) {
                Ok(()) => {
                    self.transactions.insert(id, tx);
                    Ok(())
                }
                Err(e) => Err(e),
            },
            Transaction::Dispute { id, .. } => self.dispute(id),
            Transaction::Resolve { id, .. } => self.resolve(id),
            Transaction::Chargeback { id, .. } => self.chargeback(id),
        }
    }

    fn deposit(&mut self, id: TxId, mut amount: Decimal) -> Result<(), Error> {
        if amount <= Decimal::ZERO {
            return Err(Error::AmountTooLow);
        }
        if amount.scale() > 4 {
            amount.rescale(4);
        }
        match self.available.checked_add(amount) {
            Some(amount) => self.available = amount,
            None => return Err(Error::Overflow(amount, self.available)),
        }
        self.compute_total();
        self.tx_state.insert(id, TransactionState::Normal);
        Ok(())
    }

    fn withdrawal(&mut self, mut amount: Decimal) -> Result<(), Error> {
        if amount <= Decimal::ZERO {
            return Err(Error::AmountTooLow);
        }
        if amount <= self.available {
            if amount.scale() > 4 {
                amount.rescale(4);
            }
            self.available -= amount;
            self.compute_total();
            Ok(())
        } else {
            Err(Error::InsufficientFunds(self.id))
        }
    }

    fn dispute(&mut self, id: TxId) -> Result<(), Error> {
        match self.tx_state.entry(id) {
            Entry::Vacant(..) => return Err(Error::TransactionNotFound(id, self.id)),
            Entry::Occupied(mut entry) => match entry.get() {
                TransactionState::Disputed => return Err(Error::AlreadyDisputed(id)),
                TransactionState::ResolvedOrChargedback => return Err(Error::DisputedSolved(id)),
                TransactionState::Normal => {
                    entry.insert(TransactionState::Disputed);
                }
            },
        }

        match self.transactions.get(&id) {
            Some(tx) => {
                let amount = tx.amount().unwrap();
                match self.held.checked_add(amount) {
                    Some(amount) => self.held = amount,
                    None => return Err(Error::Overflow(amount, self.held)),
                }
                self.available -= amount;
                Ok(())
            }
            None => Err(Error::TransactionNotFound(id, self.id)),
        }
    }

    fn resolve(&mut self, id: TxId) -> Result<(), Error> {
        match self.tx_state.entry(id) {
            Entry::Vacant(..) => return Err(Error::TransactionNotFound(id, self.id)),
            Entry::Occupied(mut entry) => match entry.get() {
                TransactionState::ResolvedOrChargedback => return Err(Error::DisputedSolved(id)),
                TransactionState::Normal => return Err(Error::NotInDispute(id)),
                TransactionState::Disputed => {
                    entry.insert(TransactionState::ResolvedOrChargedback);
                }
            },
        }

        match self.transactions.get(&id) {
            Some(tx) => {
                let amount = tx.amount().unwrap();
                match self.available.checked_add(amount) {
                    Some(amount) => self.available = amount,
                    None => return Err(Error::Overflow(amount, self.available)),
                }
                self.held -= amount;
                Ok(())
            }
            None => Err(Error::TransactionNotFound(id, self.id)),
        }
    }

    fn chargeback(&mut self, id: TxId) -> Result<(), Error> {
        match self.tx_state.entry(id) {
            Entry::Vacant(..) => return Err(Error::TransactionNotFound(id, self.id)),
            Entry::Occupied(mut entry) => match entry.get() {
                TransactionState::ResolvedOrChargedback => return Err(Error::DisputedSolved(id)),
                TransactionState::Normal => return Err(Error::NotInDispute(id)),
                TransactionState::Disputed => {
                    entry.insert(TransactionState::ResolvedOrChargedback);
                }
            },
        }

        match self.transactions.get(&id) {
            Some(tx) => {
                let amount = tx.amount().unwrap();
                self.held -= amount;
                self.compute_total();
                self.locked = true;
                Ok(())
            }
            None => Err(Error::TransactionNotFound(id, self.id)),
        }
    }

    fn compute_total(&mut self) {
        self.total = self.available + self.held;
    }
}

#[cfg(test)]
mod tests {
    use super::Account;
    use crate::{error::Error, transaction::TransactionState, Transaction};
    use rust_decimal::Decimal;

    // All success cases
    #[test]
    fn deposit() {
        // Setup
        let id = 1;
        let client = 1;
        let amount = Decimal::new(2, 0);
        let mut account = Account::new(client);
        let tx = Transaction::Deposit { id, client, amount };

        // Act
        let result = account.process_tx(tx);

        // Assert
        assert!(result.is_ok());
        assert_eq!(account.available, amount);
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.total, amount);
        assert!(!account.locked);
        assert!(account.transactions.contains_key(&id));
        assert!(account.tx_state.contains_key(&id));
        let state = account.tx_state.get(&id).unwrap();
        assert!(matches!(state, TransactionState::Normal));
    }

    #[test]
    fn withdrawal() {
        // Setup
        let id = 1;
        let client = 1;
        let amount = Decimal::new(2, 0);
        let mut account = Account::new(client);
        let tx = Transaction::Deposit { id, client, amount };

        assert!(account.process_tx(tx).is_ok());

        let id = 2;
        let tx = Transaction::Withdrawal { id, client, amount };
        // Act
        let result = account.process_tx(tx);

        // Assert
        assert!(result.is_ok());
        assert_eq!(account.available, Decimal::ZERO);
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.total, Decimal::ZERO);
        assert!(!account.locked);
        assert!(account.transactions.contains_key(&id));
        assert!(!account.tx_state.contains_key(&id));
    }

    #[test]
    fn dispute() {
        // Setup
        let id = 1;
        let client = 1;
        let amount = Decimal::new(2, 0);
        let mut account = Account::new(client);
        let tx = Transaction::Deposit { id, client, amount };

        assert!(account.process_tx(tx).is_ok());

        let tx = Transaction::Dispute { id, client };
        // Act
        let result = account.process_tx(tx);

        // Assert
        assert!(result.is_ok());
        assert_eq!(account.available, Decimal::ZERO);
        assert_eq!(account.held, amount);
        assert_eq!(account.total, amount);
        assert!(!account.locked);
        assert!(account.transactions.contains_key(&id));
        assert!(account.tx_state.contains_key(&id));
        let state = account.tx_state.get(&id).unwrap();
        assert!(matches!(state, TransactionState::Disputed));
    }

    #[test]
    fn resolve() {
        // Setup
        let id = 1;
        let client = 1;
        let amount = Decimal::new(2, 0);
        let mut account = Account::new(client);
        let tx = Transaction::Deposit { id, client, amount };

        assert!(account.process_tx(tx).is_ok());

        assert!(account
            .process_tx(Transaction::Dispute { id, client })
            .is_ok());
        let tx = Transaction::Resolve { id, client };

        // Act
        let result = account.process_tx(tx);

        // Assert
        assert!(result.is_ok());
        assert_eq!(account.available, amount);
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.total, amount);
        assert!(!account.locked);
        assert!(account.transactions.contains_key(&id));
        assert!(account.tx_state.contains_key(&id));
        let state = account.tx_state.get(&id).unwrap();
        assert!(matches!(state, TransactionState::ResolvedOrChargedback));
    }

    #[test]
    fn chargeback() {
        // Setup
        let id = 1;
        let client = 1;
        let amount = Decimal::new(2, 0);
        let mut account = Account::new(client);
        let tx = Transaction::Deposit { id, client, amount };

        assert!(account.process_tx(tx).is_ok());

        assert!(account
            .process_tx(Transaction::Dispute { id, client })
            .is_ok());
        let tx = Transaction::Chargeback { id, client };

        // Act
        let result = account.process_tx(tx);

        // Assert
        assert!(result.is_ok());
        assert_eq!(account.available, Decimal::ZERO);
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.total, Decimal::ZERO);
        assert!(account.locked);
        assert!(account.transactions.contains_key(&id));
        assert!(account.tx_state.contains_key(&id));
        let state = account.tx_state.get(&id).unwrap();
        assert!(matches!(state, TransactionState::ResolvedOrChargedback));
    }

    // All error cases
    #[test]
    fn unauthorized() {
        // Setup
        let id = 1;
        let client = 1;
        let amount = Decimal::new(2, 0);
        let mut account = Account::new(client);
        let client = 2;
        let tx = Transaction::Deposit { id, client, amount };

        // Act
        let result = account.process_tx(tx);

        // Assert
        assert!(matches!(result, Err(Error::Unauthorized(..))));
        assert_eq!(account.available, Decimal::ZERO);
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.total, Decimal::ZERO);
        assert!(!account.locked);
        assert!(!account.transactions.contains_key(&id));
        assert!(!account.tx_state.contains_key(&id));
    }

    #[test]
    fn locked() {
        // Setup
        let id = 1;
        let client = 1;
        let amount = Decimal::new(2, 0);
        let mut account = Account::new(client);
        let tx = Transaction::Deposit { id, client, amount };

        assert!(account.process_tx(tx).is_ok());

        assert!(account
            .process_tx(Transaction::Dispute { id, client })
            .is_ok());
        // lock account
        assert!(account
            .process_tx(Transaction::Chargeback { id, client })
            .is_ok());

        let tx = Transaction::Deposit { id, client, amount };

        // Act
        let result = account.process_tx(tx);

        // Assert
        assert!(matches!(result, Err(Error::Locked(..))));
        assert_eq!(account.available, Decimal::ZERO);
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.total, Decimal::ZERO);
        assert!(account.locked);
        assert!(account.transactions.contains_key(&id));
        assert!(account.tx_state.contains_key(&id));
        let state = account.tx_state.get(&id).unwrap();
        assert!(matches!(state, TransactionState::ResolvedOrChargedback));
    }

    #[test]
    fn amount_too_low() {
        // Setup
        let id = 1;
        let client = 1;
        let amount = Decimal::new(0, 0);
        let mut account = Account::new(client);
        let tx = Transaction::Deposit { id, client, amount };

        // Act
        let result = account.process_tx(tx);

        // Assert
        assert!(matches!(result, Err(Error::AmountTooLow)));
        assert_eq!(account.available, Decimal::ZERO);
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.total, Decimal::ZERO);
        assert!(!account.locked);
        assert!(!account.transactions.contains_key(&id));
        assert!(!account.tx_state.contains_key(&id));
    }

    #[test]
    fn insufficient_funds() {
        // Setup
        let id = 1;
        let client = 1;
        let amount = Decimal::new(2, 0);
        let mut account = Account::new(client);
        let tx = Transaction::Withdrawal { id, client, amount };

        // Act
        let result = account.process_tx(tx);

        // Assert
        assert!(matches!(result, Err(Error::InsufficientFunds(..))));
    }

    #[test]
    fn dispute_solved() {
        // Setup
        let id = 1;
        let client = 1;
        let amount = Decimal::new(2, 0);
        let mut account = Account::new(client);
        let tx = Transaction::Deposit { id, client, amount };

        assert!(account.process_tx(tx).is_ok());

        assert!(account
            .process_tx(Transaction::Dispute { id, client })
            .is_ok());
        assert!(account
            .process_tx(Transaction::Resolve { id, client })
            .is_ok());

        let tx = Transaction::Dispute { id, client };

        // Act
        let result = account.process_tx(tx);

        // Assert
        assert!(matches!(result, Err(Error::DisputedSolved(..))));
    }

    #[test]
    fn already_disputed() {
        // Setup
        let id = 1;
        let client = 1;
        let amount = Decimal::new(2, 0);
        let mut account = Account::new(client);
        let tx = Transaction::Deposit { id, client, amount };

        assert!(account.process_tx(tx).is_ok());
        assert!(account
            .process_tx(Transaction::Dispute { id, client })
            .is_ok());

        let tx = Transaction::Dispute { id, client };

        // Act
        let result = account.process_tx(tx);

        // Assert
        assert!(matches!(result, Err(Error::AlreadyDisputed(..))));
    }

    #[test]
    fn transaction_not_found() {
        // Setup
        let id = 1;
        let client = 1;
        let mut account = Account::new(client);

        let tx = Transaction::Dispute { id, client };

        // Act
        let result = account.process_tx(tx);

        // Assert
        assert!(matches!(result, Err(Error::TransactionNotFound(..))));

        let tx = Transaction::Resolve { id, client };

        // Act resolve
        let result = account.process_tx(tx);

        // Assert
        assert!(matches!(result, Err(Error::TransactionNotFound(..))));

        let tx = Transaction::Chargeback { id, client };
        // Act chargeback
        let result = account.process_tx(tx);

        assert!(matches!(result, Err(Error::TransactionNotFound(..))));
    }

    #[test]
    fn not_in_dispute() {
        // Setup
        let id = 1;
        let client = 1;
        let amount = Decimal::new(2, 0);
        let mut account = Account::new(client);

        assert!(account
            .process_tx(Transaction::Deposit { id, client, amount })
            .is_ok());

        let tx = Transaction::Resolve { id, client };
        // Act resolve
        let result = account.process_tx(tx);

        // Assert
        assert!(matches!(result, Err(Error::NotInDispute(..))));

        let tx = Transaction::Chargeback { id, client };
        // Act chargeback
        let result = account.process_tx(tx);

        assert!(matches!(result, Err(Error::NotInDispute(..))));
    }

    #[test]
    fn overflow() {
        // Setup
        let id = 1;
        let client = 1;
        let amount = Decimal::MAX;
        let mut account = Account::new(client);

        assert!(account
            .process_tx(Transaction::Deposit { id, client, amount })
            .is_ok());

        let amount = Decimal::new(2, 0);
        let id = 2;
        let tx = Transaction::Deposit { id, client, amount };

        // Act
        let result = account.process_tx(tx);

        // Assert
        assert!(matches!(result, Err(Error::Overflow(..))));
    }
}
