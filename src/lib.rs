/* to add the whole engine in here and start it through main */
/* easier to implement integration tests, benchmarks */
use std::fs::File;

use eyre::Result;
use tracing::info;

use crate::source::{csv::CsvFileTxSource, tx_source::TxSource};
mod core;
mod source;

pub struct PaymentEngine;

impl PaymentEngine {
    // add generic input source so that it's easier to test
    // update this result to produce the report so that it's easier
    // to add e2e tests
    pub async fn start_app(file: File) -> Result<()> {
        info!(?file, "starting payments engine");
        let csv_file_tx_source = CsvFileTxSource::new(file);
        let tx_stream = csv_file_tx_source.into_stream_tx();
        for tx in tx_stream {
            tracing::info!(?tx);
        }
        Ok(())
    }
}
