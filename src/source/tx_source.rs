use crate::core::types::Tx;

pub trait TxSource {
    fn into_stream_tx(self) -> impl Iterator<Item = Tx>
    where
        Self: Sized;
}
