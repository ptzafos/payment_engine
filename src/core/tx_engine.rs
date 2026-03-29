use std::marker::PhantomData;

use eyre::Context;
use futures::future::join_all;
use tokio::{sync::mpsc::Sender, task::JoinHandle};

use crate::{
    collections::Set,
    core::{
        tx_processor::TxProcessor,
        types::{Account, Tx, TxId},
    },
    source::tx_source::TxSource,
    storage::account_storage::AccountStorage,
};

const TX_PROCESSOR_BUFFER: usize = 128;

pub(crate) struct TxEngineTask<A>
where
    A: AccountStorage,
{
    pub sender: Sender<Tx>,
    pub handle: JoinHandle<TxProcessor<A>>,
}

pub struct Uninitialized;
pub struct Initialized;

pub struct TxEngine<A, S = Uninitialized>
where
    A: AccountStorage,
{
    processors: Vec<TxEngineTask<A>>,
    processed_txs: Set<TxId>,
    _state: PhantomData<S>,
}

impl<A> TxEngine<A, Uninitialized>
where
    A: AccountStorage + 'static,
{
    pub fn init(mut self) -> eyre::Result<TxEngine<A, Initialized>> {
        let n_processors = n_processors()?;
        tracing::info!(n_processors, "tx processors = num_cpus - 1");
        for n in 0..n_processors {
            self.init_processor(n);
        }
        Ok(TxEngine {
            processors: self.processors,
            processed_txs: self.processed_txs,
            _state: PhantomData,
        })
    }

    pub fn init_processor(&mut self, n_processor: usize) {
        let (tx, rx) = tokio::sync::mpsc::channel::<Tx>(TX_PROCESSOR_BUFFER);
        let tx_processor = TxProcessor::<A>::new(n_processor);
        let handle = tokio::spawn(tx_processor.spawn(rx));
        self.processors.push(TxEngineTask { sender: tx, handle });
    }
}

impl<A> TxEngine<A, Initialized>
where
    A: AccountStorage,
{
    pub async fn process_tx_source<S>(&mut self, source: S) -> eyre::Result<Vec<Account>>
    where
        S: TxSource,
    {
        assert!(
            !self.processors.is_empty(),
            "tx engine must have at least one processor"
        );

        let tx_stream = source.into_stream_tx();
        let mut dispatched_count = 0usize;
        for tx in tx_stream {
            if !self.filter_duplicate(&tx) {
                tracing::debug!(tx_id = tx.tx_id(), "duplicate create transaction ignored");
                continue;
            }
            self.dispatch_record(tx).await;
            dispatched_count += 1;
        }
        tracing::info!(
            dispatched_count,
            "all transactions dispatched to processors"
        );

        let processors = std::mem::take(&mut self.processors);
        let handles = processors.into_iter().map(|p| p.handle).collect::<Vec<_>>();

        let account_state = join_all(handles)
            .await
            .into_iter()
            .flatten()
            .flat_map(|processor| processor.account_storage.report_state())
            .collect::<Vec<_>>();

        tracing::info!(
            accounts = account_state.len(),
            "collected final account state"
        );

        Ok(account_state)
    }

    fn dispatcher_id(&self, tx: &Tx) -> usize {
        tx.client_id() as usize % self.processors.len()
    }

    async fn dispatch_record(&mut self, tx: Tx) {
        let dispatcher_id = self.dispatcher_id(&tx);
        if let Err(e) = self.processors[dispatcher_id].sender.send(tx).await {
            tracing::error!(?e, dispatcher_id, "send error on dispatcher");
        }
    }

    fn filter_duplicate(&mut self, tx: &Tx) -> bool {
        match tx {
            Tx::Deposit { .. } | Tx::Withdrawal { .. } => self.processed_txs.insert(tx.tx_id()),
            Tx::Dispute { .. } | Tx::Resolve { .. } | Tx::Chargeback { .. } => true,
        }
    }
}

impl<A, S> TxEngine<A, S>
where
    A: AccountStorage,
{
    pub fn new() -> Self {
        Self {
            processors: <_>::default(),
            processed_txs: <_>::default(),
            _state: PhantomData,
        }
    }
}

fn n_processors() -> eyre::Result<usize> {
    std::thread::available_parallelism()
        .map(|n| {
            // safe as NonZero by design
            let n = n.get();
            if n > 1 { n - 1 } else { 1 }
        })
        .with_context(|| "unable to get available threads")
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use crate::{
        collections::Set,
        core::types::{Amount, BaseTx},
        source::tx_source::TxSource,
        storage::in_memory_account::InMemoryAccountStorage,
    };

    use super::{Initialized, Tx, TxEngine};

    const CLIENT_A: u16 = 1;
    const CLIENT_B: u16 = 2;

    fn amount_units(units: i64) -> Amount {
        Amount::from_dec(rust_decimal::Decimal::new(units, 4))
    }

    fn deposit(client_id: u16, tx: u32, units: i64) -> Tx {
        Tx::Deposit {
            base: BaseTx { client_id, tx },
            amount: amount_units(units),
        }
    }

    fn withdrawal(client_id: u16, tx: u32, units: i64) -> Tx {
        Tx::Withdrawal {
            base: BaseTx { client_id, tx },
            amount: amount_units(units),
        }
    }

    fn dispute(client_id: u16, tx: u32) -> Tx {
        Tx::Dispute {
            base: BaseTx { client_id, tx },
        }
    }

    fn resolve(client_id: u16, tx: u32) -> Tx {
        Tx::Resolve {
            base: BaseTx { client_id, tx },
        }
    }

    fn sample_transactions() -> Vec<Tx> {
        vec![
            deposit(CLIENT_A, 1, 20_000),
            dispute(CLIENT_A, 1),
            resolve(CLIENT_A, 1),
            withdrawal(CLIENT_A, 2, 5_000),
            deposit(CLIENT_B, 3, 10_000),
            deposit(CLIENT_B, 3, 99_999),
        ]
    }

    struct VecTxSource {
        txs: Vec<Tx>,
    }

    impl TxSource for VecTxSource {
        fn into_stream_tx(self) -> impl Iterator<Item = Tx>
        where
            Self: Sized,
        {
            self.txs.into_iter()
        }
    }

    #[test]
    fn filters_duplicate_create_transactions_only() {
        let mut engine = TxEngine::<InMemoryAccountStorage, Initialized> {
            processors: Vec::new(),
            processed_txs: Set::default(),
            _state: PhantomData,
        };

        let create = deposit(CLIENT_A, 11, 10_000);
        let dispute = dispute(CLIENT_A, 11);

        assert!(engine.filter_duplicate(&create));
        assert!(!engine.filter_duplicate(&create));
        assert!(engine.filter_duplicate(&dispute));
    }

    #[tokio::test]
    async fn processes_source_end_to_end() {
        let source = VecTxSource {
            txs: sample_transactions(),
        };

        let mut engine = TxEngine::<InMemoryAccountStorage>::new()
            .init()
            .expect("engine init should succeed");

        let mut accounts = engine
            .process_tx_source(source)
            .await
            .expect("processing should succeed");
        accounts.sort_by_key(|account| account.client_id);

        assert_eq!(accounts.len(), 2);
        assert_eq!(accounts[0].client_id, CLIENT_A);
        assert_eq!(*accounts[0].available, 15_000);
        assert_eq!(*accounts[0].held, 0);
        assert_eq!(*accounts[0].total, 15_000);
        assert!(!accounts[0].locked);

        assert_eq!(accounts[1].client_id, CLIENT_B);
        assert_eq!(*accounts[1].available, 10_000);
        assert_eq!(*accounts[1].held, 0);
        assert_eq!(*accounts[1].total, 10_000);
        assert!(!accounts[1].locked);
    }

    #[test]
    fn n_processors_is_at_least_one() {
        let processors = super::n_processors().expect("should detect processor count");
        assert!(processors >= 1);
    }
}
