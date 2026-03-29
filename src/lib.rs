/* to add the whole engine in here and start it through main */
/* easier to implement integration tests, benchmarks */
use std::fs::File;
use tracing::info;

use crate::{
    core::tx_engine::TxEngine,
    report::Report,
    source::csv::CsvFileTxSource,
    storage::{
        account::in_memory_account::InMemoryAccountStorage, tx::in_memory_tx::InMemoryTxStorage,
    },
};
mod core;
mod report;
mod source;
mod storage;

pub struct PaymentEngine;

impl PaymentEngine {
    pub async fn start_app(file: File) -> eyre::Result<()> {
        info!(?file, "starting payments engine");
        let csv_file_tx_source = CsvFileTxSource::new(file);
        let tx_engine = TxEngine::<InMemoryTxStorage, InMemoryAccountStorage>::new();
        let mut tx_engine = tx_engine.init()?;
        let report = tx_engine.process_tx_source(csv_file_tx_source).await?;
        Report::new(report).write_stdout()?;
        Ok(())
    }
}
