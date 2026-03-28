use std::marker::PhantomData;

use eyre::Context;
use tokio::{sync::mpsc::Sender, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use crate::{
    core::{tx_processor::TxProcessor, types::Tx},
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
    _state: PhantomData<S>, // add metrics for hot accounts
}

struct TxEngineTask<T, A>
where
    T: TxStorage,
{
    sender: Sender<Tx>,
    handle: JoinHandle<(T, A)>,
    c_token: CancellationToken,
}

impl<T, A> TxEngine<T, A>
where
    T: TxStorage,
    A: AccountStorage,
{
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
            let (tx, rx) = tokio::sync::mpsc::channel::<Tx>(tx_processor_buffer());
            let tx_processor = TxProcessor::<T, A>::new(n);
            let c_token = CancellationToken::new();
            let handle = tokio::spawn(tx_processor.spawn(rx, c_token.clone()));
            self.processors.push(TxEngineTask {
                sender: tx,
                handle,
                c_token,
            });
        }
        Ok(TxEngine {
            processors: self.processors,
            _state: PhantomData,
        })
    }
}

impl<T, A> TxEngine<T, A, Initialized>
where
    T: TxStorage,
    A: AccountStorage,
{
    pub async fn process_tx_source<S>(&mut self, source: S) -> eyre::Result<()>
    where
        S: TxSource,
    {
        let tx_stream = source.into_stream_tx();
        for tx in tx_stream {
            self.dispatch_record(tx).await;
        }
        Ok(())
    }

    fn routing_id(&self, tx: &Tx) -> usize {
        tx.client_id() as usize % self.processors.len()
    }

    async fn dispatch_record(&mut self, tx: Tx) {
        let dispatcher_id = self.routing_id(&tx);
        // to remove this copy no reason for only the log maybe?
        let tx_id = tx.tx_id();
        if let Err(e) = self.processors[dispatcher_id].sender.send(tx).await {
            tracing::error!(?e, "unable to process tx with id {}", tx_id);
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

fn tx_processor_buffer() -> usize {
    TX_PROCESSOR_BUFFER
}
