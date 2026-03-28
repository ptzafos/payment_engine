/* to add the whole engine in here and start it through main */
/* easier to implement integration tests, benchmarks */
use std::fs::File;
use tracing::info;

use crate::{
    core::tx_engine::TxEngine,
    source::csv::CsvFileTxSource,
    storage::{
        account::in_memory_account::InMemoryAccountStorage, tx::in_memory_tx::InMemoryTxStorage,
    },
};
mod core;
mod source;
mod storage;

pub struct PaymentEngine;

impl PaymentEngine {
    // add generic input source so that it's easier to test
    // update this result to produce the report so that it's easier
    // to add e2e tests
    pub async fn start_app(file: File) -> eyre::Result<()> {
        info!(?file, "starting payments engine");
        let csv_file_tx_source = CsvFileTxSource::new(file);
        let tx_engine = TxEngine::<InMemoryTxStorage, InMemoryAccountStorage>::new();
        let mut tx_engine = tx_engine.init()?;
        tx_engine.process_tx_source(csv_file_tx_source).await?;
        // let report = tx_engine.report_state();
        Ok(())
    }
}
