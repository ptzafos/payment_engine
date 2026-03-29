use std::ops::{Add, AddAssign, Deref, Sub, SubAssign};

use rust_decimal::{Decimal, prelude::ToPrimitive};
use serde::{Deserialize, Serialize};

use crate::collections::Map;

pub const SCALING: i64 = 10_000;

pub type ClientId = u16;
pub type TxId = u32;

#[derive(Deserialize, Debug, Default)]
pub struct BaseTx {
    pub client_id: ClientId,
    pub tx: TxId,
}

impl BaseTx {
    pub fn new(client: ClientId, tx: TxId) -> Self {
        Self {
            client_id: client,
            tx,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Amount(i64);

impl Amount {
    pub fn from_dec(dec_amount: Decimal) -> Self {
        let scaled = dec_amount
            .checked_mul(Decimal::from(SCALING))
            .expect("amount overflow while scaling");

        let units = scaled.to_i64().expect("amount is out of i64 range");

        Self(units)
    }
}

impl Add for Amount {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for Amount {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl AddAssign for Amount {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl SubAssign for Amount {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
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
    pub fn client_id(&self) -> ClientId {
        match self {
            Tx::Deposit { base, .. }
            | Tx::Withdrawal { base, .. }
            | Tx::Dispute { base }
            | Tx::Resolve { base }
            | Tx::Chargeback { base } => base.client_id,
        }
    }
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct Account {
    pub client_id: ClientId,
    pub available: Amount,
    pub held: Amount,
    pub total: Amount,
    pub locked: bool,
}

#[derive(Debug, Default)]
pub struct AccountState {
    pub account: Account,
    pub deposits: Map<TxId, Amount>,
    pub open_disputes: Map<TxId, Amount>,
}
