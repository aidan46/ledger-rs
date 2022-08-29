use crate::{ClientId, TxId};
use rust_decimal::Decimal;

#[derive(Debug)]
pub enum Transaction {
    Deposit {
        id: TxId,
        client: ClientId,
        amount: Decimal,
    },
    Withdrawal {
        id: TxId,
        client: ClientId,
        amount: Decimal,
    },
    Dispute {
        id: TxId,
        client: ClientId,
    },
    Resolve {
        id: TxId,
        client: ClientId,
    },
    Chargeback {
        id: TxId,
        client: ClientId,
    },
}

#[derive(Debug)]
pub(crate) enum TransactionState {
    Normal,
    Disputed,
    ResolvedOrChargedback,
}

impl Transaction {
    pub fn id(&self) -> TxId {
        match *self {
            Self::Deposit { id, .. }
            | Self::Withdrawal { id, .. }
            | Self::Dispute { id, .. }
            | Self::Resolve { id, .. }
            | Self::Chargeback { id, .. } => id,
        }
    }

    pub fn client(&self) -> ClientId {
        match *self {
            Self::Deposit { client, .. }
            | Self::Withdrawal { client, .. }
            | Self::Dispute { client, .. }
            | Self::Resolve { client, .. }
            | Self::Chargeback { client, .. } => client,
        }
    }

    pub fn amount(&self) -> Option<Decimal> {
        match *self {
            Self::Deposit { amount, .. } | Self::Withdrawal { amount, .. } => Some(amount),
            _ => None,
        }
    }
}
