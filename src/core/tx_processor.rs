use tokio::{select, sync::mpsc::Receiver};
use tokio_util::sync::CancellationToken;

use crate::{
    core::types::Tx,
    storage::{account::account_storage::AccountStorage, tx::tx_storage::TxStorage},
};

pub(crate) struct TxProcessor<T, A>
where
    T: TxStorage,
    A: AccountStorage,
{
    id: usize,
    tx_storage: T,
    account_storage: A,
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

    pub async fn spawn(self, mut rx: Receiver<Tx>, c_token: CancellationToken) -> (T, A) {
        loop {
            select! {
                msg = rx.recv() => {
                    match msg {
                        Some(tx) => {
                            tracing::info!(?tx);
                        }
                        None => {

                        }
                    }
                    //validate op
                    //apply changes
                }
                () = c_token.cancelled() => {
                    tracing::info!(self.id, "tx processor cancelled");
                    break;
                }
            }
        }
        (self.tx_storage, self.account_storage)
    }
}
