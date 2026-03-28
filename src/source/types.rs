use eyre::ContextCompat;
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::core::types::{Amount, BaseTx, ClientId, Tx, TxId};

#[derive(Deserialize)]
pub(crate) struct TxCsvRow {
    r#type: TxCsvTypeRow,
    client: ClientId,
    tx: TxId,
    // TODO Custom deserializer to do String to Decimal or i128
    amount: Option<Decimal>,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum TxCsvTypeRow {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

impl TryFrom<TxCsvRow> for Tx {
    type Error = eyre::Error;

    fn try_from(value: TxCsvRow) -> eyre::Result<Self> {
        let base = BaseTx::new(value.client, value.tx);
        Ok(match value.r#type {
            TxCsvTypeRow::Deposit => Tx::Deposit {
                amount: Amount::from_dec(value.amount.with_context(|| {
                    format!("amount not found for deposit with tx {}", base.tx)
                })?),
                base,
            },
            TxCsvTypeRow::Withdrawal => Tx::Withdrawal {
                amount: Amount::from_dec(value.amount.with_context(|| {
                    format!("amount not found for withdrawal with tx {}", base.tx)
                })?),
                base,
            },
            TxCsvTypeRow::Dispute => Tx::Dispute { base },
            TxCsvTypeRow::Resolve => Tx::Resolve { base },
            TxCsvTypeRow::Chargeback => Tx::Chargeback { base },
        })
    }
}
