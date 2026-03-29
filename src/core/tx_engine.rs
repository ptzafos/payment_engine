use std::marker::PhantomData;

use eyre::Context;
use futures::future::join_all;
use tokio_util::sync::CancellationToken;

use crate::{
    core::{
        tx_engine_task::TxEngineTask,
        tx_processor::TxProcessor,
        types::{Account, Tx},
    },
    source::tx_source::TxSource,
    storage::{account::account_storage::AccountStorage, tx::tx_storage::TxStorage},
};

const TX_PROCESSOR_BUFFER: usize = 128;

pub struct Uninitialized;
pub struct Initialized;

pub struct TxEngine<T, A, S = Uninitialized>
where
    T: TxStorage,
    A: AccountStorage,
{
    processors: Vec<TxEngineTask<T, A>>,
    _state: PhantomData<S>,
}

impl<T, A> TxEngine<T, A, Uninitialized>
where
    T: TxStorage + 'static,
    A: AccountStorage + 'static,
{
    pub fn init(mut self) -> eyre::Result<TxEngine<T, A, Initialized>> {
        let n_tx_processors = n_processors()?;
        tracing::info!(n_tx_processors, "tx processors = num_cpus - 1");
        for n in 0..n_tx_processors {
            self.init_processor(n);
        }
        Ok(TxEngine {
            processors: self.processors,
            _state: PhantomData,
        })
    }

    pub fn init_processor(&mut self, n_processor: usize) {
        let (tx, rx) = tokio::sync::mpsc::channel::<Tx>(TX_PROCESSOR_BUFFER);
        let tx_processor = TxProcessor::<T, A>::new(n_processor);
        let stop_token = CancellationToken::new();
        let handle = tokio::spawn(tx_processor.spawn(rx, stop_token.clone()));
        self.processors.push(TxEngineTask {
            sender: tx,
            handle,
            stop_token,
        });
    }
}

impl<T, A> TxEngine<T, A, Initialized>
where
    T: TxStorage,
    A: AccountStorage,
{
    pub async fn process_tx_source<S>(&mut self, source: S) -> eyre::Result<Vec<Account>>
    where
        S: TxSource,
    {
        let tx_stream = source.into_stream_tx();
        for tx in tx_stream {
            self.dispatch_record(tx).await;
        }
        self.processors.iter_mut().for_each(|p| {
            // std::mem::take(&mut p.sender);
            p.stop_token.cancel();
        });
        let processors = std::mem::take(&mut self.processors);
        let report = join_all(processors.into_iter().map(|p| p.handle).collect::<Vec<_>>())
            .await
            .into_iter()
            .flatten()
            .flat_map(|processor| processor.account_storage.report_state())
            .collect::<Vec<_>>();
        Ok(report)
    }

    fn routing_id(&self, tx: &Tx) -> usize {
        tx.client_id() as usize % self.processors.len()
    }

    async fn dispatch_record(&mut self, tx: Tx) {
        let dispatcher_id = self.routing_id(&tx);
        if let Err(e) = self.processors[dispatcher_id].sender.send(tx).await {
            tracing::error!(?e, dispatcher_id, "send error on dispatcher");
        }
    }
}

impl<T, A, S> TxEngine<T, A, S>
where
    T: TxStorage,
    A: AccountStorage,
{
    pub fn new() -> Self {
        Self {
            processors: <_>::default(),
            _state: PhantomData,
        }
    }
}

fn n_processors() -> eyre::Result<usize> {
    // can use num_cpus crate if needed
    std::thread::available_parallelism()
        .map(|n| {
            // safe as NonZero by design
            let n = n.get();
            if n > 1 { n - 1 } else { 1 }
        })
        .with_context(|| "unable to get available threads")
}
