use crate::{ClientId, TxId};
use rust_decimal::Decimal;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Client with ID {0} is unauthorized to access account with ID {1}!")]
    Unauthorized(ClientId, ClientId),
    #[error("Account with ID {0} is locked!")]
    Locked(ClientId),
    #[error("Overflow while trying to add {0} to {1}!")]
    Overflow(Decimal, Decimal),
    #[error("Account with ID {0} has insufficient funds!")]
    InsufficientFunds(ClientId),
    #[error("Transaction with ID {0} not found for account with ID {1}!")]
    TransactionNotFound(TxId, ClientId),
    #[error("Transaction with ID {0} is not in dispute!")]
    NotInDispute(TxId),
    #[error("Transaction with ID {0} has been or is already disputed!")]
    AlreadyDisputed(TxId),
    #[error("Amount has to be above zero")]
    AmountTooLow,
    #[error("Transaction with ID {0} already exists!")]
    DuplicateTxId(TxId),
    #[error("Transaction with ID {0} has already been charged back")]
    AlreadyChargedBack(TxId),
}
