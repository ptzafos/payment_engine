use tokio::{sync::mpsc::Sender, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use crate::{
    core::{tx_processor::TxProcessor, types::Tx},
    storage::{account::account_storage::AccountStorage, tx::tx_storage::TxStorage},
};

pub(crate) struct TxEngineTask<T, A>
where
    T: TxStorage,
    A: AccountStorage,
{
    pub sender: Sender<Tx>,
    pub handle: JoinHandle<TxProcessor<T, A>>,
    pub stop_token: CancellationToken,
}
