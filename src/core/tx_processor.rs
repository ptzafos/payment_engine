use tokio::{select, sync::mpsc::Receiver};
use tokio_util::sync::CancellationToken;

use crate::{
    core::{ops::AccountOps, types::Tx},
    storage::{account::account_storage::AccountStorage, tx::tx_storage::TxStorage},
};

pub(crate) struct TxProcessor<T, A>
where
    T: TxStorage,
    A: AccountStorage,
{
    id: usize,
    tx_storage: T,
    pub account_storage: A,
}

impl<T, A> TxProcessor<T, A>
where
    T: TxStorage,
    A: AccountStorage,
{
    pub fn new(processor_id: usize) -> Self {
        Self {
            tx_storage: <_>::default(),
            account_storage: <_>::default(),
            id: processor_id,
        }
    }

    pub async fn spawn(mut self, mut rx: Receiver<Tx>, stop_token: CancellationToken) -> Self {
        loop {
            select! {
                msg = rx.recv() => {
                    match msg {
                        Some(tx) => {
                            self.process_tx(tx);
                        }
                        None => {
                            tracing::info!(?self.id, "recv None, channel closed for dispatcher");
                            break;
                        }
                    }
                }
                () = stop_token.cancelled() => {
                    tracing::info!(self.id, "stopping tx processor");
                    while let Some(tx) = rx.recv().await {
                       self.process_tx(tx);
                    }
                    break;
                }
            }
        }
        self
    }

    fn process_tx(&mut self, tx: Tx) {
        if self.tx_storage.contains(&tx) {
            tracing::error!(?tx, "transaction already exists");
        }
        let account = self.account_storage.get_account_by_id(&tx.client_id());
        tracing::debug!(?tx, ?account, "processing tx for account");
        self.tx_storage.save(tx);
    }
}
