use crate::core::types::{Tx, TxId};

pub trait TxStorage: Default + Send {
    fn save(&mut self, tx: Tx);
    fn get_by_id(&mut self, tx_id: &TxId) -> Option<&Tx>;
}
