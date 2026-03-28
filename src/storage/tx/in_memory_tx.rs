use crate::{
    core::types::{Tx, TxId},
    storage::tx::tx_storage::TxStorage,
};

#[derive(Default)]
pub struct InMemoryTxStorage {
    storage: hashbrown::HashMap<TxId, Tx>,
}

impl TxStorage for InMemoryTxStorage {
    fn save(&mut self, tx: Tx) {
        self.storage.insert(tx.tx_id(), tx);
    }

    fn get_by_id(&mut self, tx_id: &TxId) -> Option<&Tx> {
        self.storage.get(tx_id)
    }
}
