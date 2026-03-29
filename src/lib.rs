use std::fs::File;

use crate::{
    core::{tx_engine::TxEngine, types::Account},
    report::{AccountSink, CsvStdoutSink},
    source::csv::CsvFileTxSource,
    storage::in_memory_account::InMemoryAccountStorage,
};

mod collections;
mod core;
mod report;
mod source;
mod storage;

pub struct PaymentEngine;

impl PaymentEngine {
    pub async fn start_app(file: File) -> eyre::Result<Vec<Account>> {
        let source = CsvFileTxSource::new(file);
        let tx_engine = TxEngine::<InMemoryAccountStorage>::new();
        let tx_engine = tx_engine.init()?;
        let accounts = tx_engine.process_tx_source(source).await?;
        CsvStdoutSink.write_accounts(&accounts)?;
        Ok(accounts)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, File},
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::PaymentEngine;

    fn temp_csv_path() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "payments_engine_e2e_{}_{}.csv",
            std::process::id(),
            nanos
        ))
    }

    #[tokio::test]
    async fn runs_end_to_end_with_dispute_flow() {
        let path = temp_csv_path();
        let input = "type,client,tx,amount\n\
deposit,1,1,2.0\n\
deposit,2,2,3.0\n\
withdrawal,2,3,1.5\n\
dispute,1,1,\n\
resolve,1,1,\n\
dispute,2,2,\n\
chargeback,2,2,\n";

        fs::write(&path, input).expect("csv input should be written");

        let file = File::open(&path).expect("csv input file should open");
        let mut accounts = PaymentEngine::start_app(file)
            .await
            .expect("payment engine should process input");
        accounts.sort_by_key(|account| account.client_id);

        assert_eq!(accounts.len(), 2);

        assert_eq!(accounts[0].client_id, 1);
        assert_eq!(*accounts[0].available, 20_000);
        assert_eq!(*accounts[0].held, 0);
        assert_eq!(*accounts[0].total, 20_000);
        assert!(!accounts[0].locked);

        assert_eq!(accounts[1].client_id, 2);
        assert_eq!(*accounts[1].available, -15_000);
        assert_eq!(*accounts[1].held, 0);
        assert_eq!(*accounts[1].total, -15_000);
        assert!(accounts[1].locked);

        let _ = fs::remove_file(path);
    }
}
