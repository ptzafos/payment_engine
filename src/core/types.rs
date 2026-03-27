use std::ops::Deref;

use rust_decimal::{Decimal, prelude::ToPrimitive};
use serde::{Deserialize, Serialize};

pub const SCALING: i64 = 10_000;

#[derive(Deserialize, Debug, Default)]
pub struct BaseTx {
    pub client: u16,
    pub tx: u32,
}

impl BaseTx {
    pub fn new(client: u16, tx: u32) -> Self {
        Self { client, tx }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, Default)]
pub struct Amount(i64);

impl Amount {
    pub fn from_dec(dec_amount: Decimal) -> Self {
        Amount(
            dec_amount
                .to_i64()
                .expect("to dec deserialization promised")
                * SCALING,
        )
    }
}

impl Deref for Amount {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Deserialize, Debug)]
pub enum Tx {
    Deposit { base: BaseTx, amount: Amount },
    Withdrawal { base: BaseTx, amount: Amount },
    Dispute { base: BaseTx },
    Resolve { base: BaseTx },
    Chargeback { base: BaseTx },
}

impl Tx {
    fn client_id(&self) -> u16 {
        match self {
            Tx::Deposit { base, .. }
            | Tx::Withdrawal { base, .. }
            | Tx::Dispute { base, .. }
            | Tx::Resolve { base, .. }
            | Tx::Chargeback { base, .. } => base.client,
        }
    }

    fn tx_id(&self) -> u32 {
        match self {
            Tx::Deposit { base, .. }
            | Tx::Withdrawal { base, .. }
            | Tx::Dispute { base, .. }
            | Tx::Resolve { base, .. }
            | Tx::Chargeback { base, .. } => base.tx,
        }
    }

    fn amount(&self) -> Option<Amount> {
        match self {
            Tx::Deposit { amount, .. } | Tx::Withdrawal { amount, .. } => Some(*amount),
            _ => None,
        }
    }
}

#[derive(Serialize, Debug)]
struct Account {
    available: Amount,
    held: Amount,
    total: Amount,
    locked: bool,
}
