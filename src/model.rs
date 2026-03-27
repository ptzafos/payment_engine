use std::{default, ops::Deref};

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Default)]
struct BaseTransaction {
    client: u16,
    tx: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, Default)]
struct Amount(NumRepr);

impl Deref for Amount {
    type Target = NumRepr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

type NumRepr = Decimal;

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "lowercase")]
enum Transaction {
    Deposit {
        #[serde(flatten)]
        base: BaseTransaction,
        amount: Amount,
    },
    Withdrawal {
        #[serde(flatten)]
        base: BaseTransaction,
        amount: Amount,
    },
    Dispute {
        #[serde(flatten)]
        base: BaseTransaction,
    },
    Resolve {
        #[serde(flatten)]
        base: BaseTransaction,
    },
    Chargeback {
        #[serde(flatten)]
        base: BaseTransaction,
    },
}

impl Transaction {
    fn client_id(&self) -> u16 {
        match self {
            Transaction::Deposit { base, .. }
            | Transaction::Withdrawal { base, .. }
            | Transaction::Dispute { base, .. }
            | Transaction::Resolve { base, .. }
            | Transaction::Chargeback { base, .. } => base.client,
        }
    }

    fn tx_id(&self) -> u32 {
        match self {
            Transaction::Deposit { base, .. }
            | Transaction::Withdrawal { base, .. }
            | Transaction::Dispute { base, .. }
            | Transaction::Resolve { base, .. }
            | Transaction::Chargeback { base, .. } => base.tx,
        }
    }

    fn amount(&self) -> Option<Amount> {
        match self {
            Transaction::Deposit { amount, .. } | Transaction::Withdrawal { amount, .. } => {
                Some(*amount)
            }
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

#[cfg(test)]
mod test {
    use csv::{ReaderBuilder, Trim};

    use crate::model::Transaction;

    #[test]
    #[ignore = "to be fixed"]
    fn deserialization_test() {
        let data = r#"type, client, tx, amount
            deposit, 1, 1, 1.0
            deposit, 2, 2, 2.0
            dispute, 1, 3,
            resolve, 1, 4,
            chargeback, 2, 5,
            "#;

        let data = data
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n");

        let mut rdr = ReaderBuilder::new()
            .trim(Trim::All)
            .from_reader(data.as_bytes());

        for row in rdr.deserialize() {
            let tx: Transaction = row.expect("deserialization should not fail");
            println!("{tx:?}");
        }
    }
}
