mod account;
mod error;
mod transaction;

pub type ClientId = u16;
pub type TxId = u32;

pub use transaction::Transaction;
