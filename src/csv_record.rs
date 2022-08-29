use ledger_rs::{ClientId, Transaction, TxId};
use rust_decimal::Decimal;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub(crate) struct TransactionRecord {
    #[serde(rename(deserialize = "type"))]
    pub tx_type: TxType,
    pub client: ClientId,
    pub tx: TxId,
    #[serde(deserialize_with = "csv::invalid_option")]
    pub amount: Option<Decimal>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum TxType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Error)]
#[error("Missing amount")]
pub struct RecordError;

impl TryFrom<TransactionRecord> for Transaction {
    type Error = RecordError;
    fn try_from(record: TransactionRecord) -> Result<Self, Self::Error> {
        let id = record.tx;
        let client = record.client;
        match record.tx_type {
            TxType::Deposit => match record.amount {
                Some(amount) => Ok(Self::Deposit { id, client, amount }),
                None => Err(RecordError),
            },
            TxType::Withdrawal => match record.amount {
                Some(amount) => Ok(Self::Withdrawal { id, client, amount }),
                None => Err(RecordError),
            },
            TxType::Dispute => Ok(Self::Dispute { id, client }),
            TxType::Resolve => Ok(Self::Resolve { id, client }),
            TxType::Chargeback => Ok(Self::Chargeback { id, client }),
        }
    }
}
