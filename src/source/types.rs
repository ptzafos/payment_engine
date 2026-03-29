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

#[cfg(test)]
mod tests {
    use csv::ReaderBuilder;

    use super::{Tx, TxCsvRow};

    #[test]
    fn parses_deposit_row_into_transaction() {
        let input = "type,client,tx,amount\ndeposit,1,10,1.2500\n";
        let mut reader = ReaderBuilder::new().from_reader(input.as_bytes());
        let row = reader
            .deserialize::<TxCsvRow>()
            .next()
            .expect("row should exist")
            .expect("csv row should deserialize");

        let tx = Tx::try_from(row).expect("tx conversion should succeed");

        match tx {
            Tx::Deposit { base, amount } => {
                assert_eq!(base.client_id, 1);
                assert_eq!(base.tx, 10);
                assert_eq!(*amount, 12_500);
            }
            _ => panic!("expected deposit transaction"),
        }
    }

    #[test]
    fn fails_when_deposit_amount_is_missing() {
        let input = "type,client,tx,amount\ndeposit,1,10,\n";
        let mut reader = ReaderBuilder::new().from_reader(input.as_bytes());
        let row = reader
            .deserialize::<TxCsvRow>()
            .next()
            .expect("row should exist")
            .expect("csv row should deserialize");

        let result = Tx::try_from(row);

        assert!(result.is_err());
    }
}
