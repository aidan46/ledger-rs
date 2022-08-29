pub mod error;

mod account;
mod ledger;
mod transaction;

pub type ClientId = u16;
pub type TxId = u32;

pub use ledger::Ledger;
pub use transaction::Transaction;
