use std::io::{self, Write};

use rust_decimal::Decimal;
use serde::Serialize;

use crate::core::types::{Account, Amount, SCALING};

pub trait AccountSink {
    fn write_accounts(&self, accounts: &[Account]) -> eyre::Result<()>;
}

pub struct CsvStdoutSink;

impl AccountSink for CsvStdoutSink {
    fn write_accounts(&self, accounts: &[Account]) -> eyre::Result<()> {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        write_csv_accounts(accounts, &mut handle)?;
        Ok(())
    }
}

fn write_csv_accounts<W: Write>(accounts: &[Account], writer: W) -> csv::Result<()> {
    let mut csv_writer = csv::WriterBuilder::new()
        .has_headers(true)
        .from_writer(writer);

    for account in accounts {
        let row = AccountCsvRow::from(account);
        csv_writer.serialize(row)?;
    }

    csv_writer.flush()?;
    Ok(())
}

#[derive(Serialize)]
struct AccountCsvRow {
    client: u16,
    available: String,
    held: String,
    total: String,
    locked: bool,
}

impl From<&Account> for AccountCsvRow {
    fn from(value: &Account) -> Self {
        Self {
            client: value.client_id,
            available: format_amount(value.available),
            held: format_amount(value.held),
            total: format_amount(value.total),
            locked: value.locked,
        }
    }
}

fn format_amount(amount: Amount) -> String {
    Decimal::new(*amount, SCALING.trailing_zeros())
        .normalize()
        .to_string()
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;

    use super::{format_amount, write_csv_accounts, Account, AccountCsvRow, Amount};

    fn amount(v: Decimal) -> Amount {
        Amount::from_dec(v)
    }

    #[test]
    fn formats_amount_with_trimmed_trailing_zeros() {
        assert_eq!(format_amount(amount(Decimal::new(15000, 4))), "1.5");
        assert_eq!(format_amount(amount(Decimal::new(20000, 4))), "2");
    }

    #[test]
    fn writes_csv_header_and_rows() {
        let accounts = vec![Account {
            client_id: 7,
            available: amount(Decimal::new(25000, 4)),
            held: amount(Decimal::new(0, 0)),
            total: amount(Decimal::new(25000, 4)),
            locked: false,
        }];

        let mut out = Vec::new();
        write_csv_accounts(&accounts, &mut out).expect("csv write should succeed");

        let text = String::from_utf8(out).expect("output should be valid utf-8");
        assert!(text.starts_with("client,available,held,total,locked\n"));
        assert!(text.contains("7,2.5,0,2.5,false"));
    }

    #[test]
    fn row_projection_uses_formatted_amounts() {
        let account = Account {
            client_id: 3,
            available: amount(Decimal::new(12340, 4)),
            held: amount(Decimal::new(10, 1)),
            total: amount(Decimal::new(22340, 4)),
            locked: true,
        };

        let row = AccountCsvRow::from(&account);
        assert_eq!(row.client, 3);
        assert_eq!(row.available, "1.234");
        assert_eq!(row.held, "1");
        assert_eq!(row.total, "2.234");
        assert!(row.locked);
    }
}
