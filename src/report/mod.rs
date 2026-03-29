use std::io::{self, Write};

use rust_decimal::Decimal;
use serde::Serialize;

use crate::core::types::{Account, Amount, SCALING};

pub struct Report {
    accounts: Vec<Account>,
}

impl Report {
    pub fn new(accounts: Vec<Account>) -> Self {
        Self { accounts }
    }

    pub fn write_stdout(&self) -> csv::Result<()> {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        self.write_to(&mut handle)
    }

    fn write_to<W: Write>(&self, writer: W) -> csv::Result<()> {
        let mut csv_writer = csv::WriterBuilder::new()
            .has_headers(true)
            .from_writer(writer);

        for account in &self.accounts {
            let row = AccountCsvRow::from(account);
            csv_writer.serialize(row)?;
        }

        csv_writer.flush()?;
        Ok(())
    }
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
