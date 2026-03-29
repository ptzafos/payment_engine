use std::fs::File;

use csv::{Reader, ReaderBuilder};

use crate::{
    core::types::Tx,
    source::{tx_source::TxSource, types::TxCsvRow},
};

pub struct CsvFileTxSource {
    reader: Reader<File>,
}

impl CsvFileTxSource {
    pub fn new(file: File) -> Self {
        let reader = ReaderBuilder::new()
            .has_headers(true)
            .trim(csv::Trim::All)
            .from_reader(file);
        Self { reader }
    }
}

impl TxSource for CsvFileTxSource {
    fn into_stream_tx(self) -> impl Iterator<Item = Tx>
    where
        Self: Sized,
    {
        let iter = self.reader.into_deserialize::<TxCsvRow>();
        iter.filter_map(parse)
    }
}

fn parse(csv_row_result: Result<TxCsvRow, csv::Error>) -> Option<Tx> {
    let row = match csv_row_result {
        Ok(row) => row,
        Err(e) => {
            tracing::error!(?e, "Unable to parse row");
            return None;
        }
    };

    row.try_into()
        .map_err(|e| {
            tracing::error!(?e, "Unable to convert csv row to transaction");
            e
        })
        .ok()
}
