use tokio::sync::mpsc::Receiver;

use crate::{
    core::types::{Amount, ClientId, Tx, TxId},
    storage::account_storage::AccountStorage,
};

pub(crate) struct TxProcessor<A>
where
    A: AccountStorage,
{
    id: usize,
    pub account_storage: A,
}

impl<A> TxProcessor<A>
where
    A: AccountStorage,
{
    pub fn new(processor_id: usize) -> Self {
        Self {
            account_storage: <_>::default(),
            id: processor_id,
        }
    }

    pub async fn spawn(mut self, mut rx: Receiver<Tx>) -> Self {
        while let Some(tx) = rx.recv().await {
            self.process_tx(tx);
        }
        tracing::info!(processor_id = self.id, "tx processor stopped");
        self
    }

    fn process_tx(&mut self, tx: Tx) {
        match tx {
            Tx::Deposit { base, amount } => self.process_deposit(base.client_id, base.tx, amount),
            Tx::Withdrawal { base, amount } => {
                self.process_withdrawal(base.client_id, base.tx, amount)
            }
            Tx::Dispute { base } => self.process_dispute(base.client_id, base.tx),
            Tx::Resolve { base } => self.process_resolve(base.client_id, base.tx),
            Tx::Chargeback { base } => self.process_chargeback(base.client_id, base.tx),
        }
    }

    fn process_deposit(&mut self, client_id: ClientId, tx_id: TxId, amount: Amount) {
        let state = self.account_storage.get_state_by_id_mut(client_id);

        if state.deposits.contains_key(&tx_id) {
            tracing::debug!(client_id, tx_id, "duplicate deposit ignored");
            return;
        }

        if let Err(err) = state.account.apply_deposit(amount) {
            tracing::debug!(?err, client_id, tx_id, "ignoring deposit");
            return;
        }

        state.deposits.insert(tx_id, amount);
    }

    fn process_withdrawal(&mut self, client_id: ClientId, _tx_id: TxId, amount: Amount) {
        let state = self.account_storage.get_state_by_id_mut(client_id);
        if let Err(err) = state.account.try_apply_withdrawal(amount) {
            tracing::debug!(?err, client_id, "ignoring withdrawal");
        }
    }

    fn process_dispute(&mut self, client_id: ClientId, tx_id: TxId) {
        let state = self.account_storage.get_state_by_id_mut(client_id);

        if state.open_disputes.contains_key(&tx_id) {
            tracing::debug!(tx_id, client_id, "dispute already open");
            return;
        }

        let Some(&amount) = state.deposits.get(&tx_id) else {
            tracing::debug!(
                tx_id,
                client_id,
                "referenced tx not found for dispute for client id"
            );
            return;
        };

        if let Err(err) = state.account.apply_dispute(amount) {
            tracing::debug!(?err, client_id, tx_id, "ignoring dispute");
            return;
        }

        state.open_disputes.insert(tx_id, amount);
        tracing::info!(processor_id = self.id, client_id, tx_id, "dispute opened");
    }

    fn process_resolve(&mut self, client_id: ClientId, tx_id: TxId) {
        let state = self.account_storage.get_state_by_id_mut(client_id);

        let Some(&amount) = state.open_disputes.get(&tx_id) else {
            tracing::debug!(tx_id, client_id, "no open dispute found");
            return;
        };

        if let Err(err) = state.account.apply_resolve(amount) {
            tracing::debug!(?err, client_id, tx_id, "ignoring resolve");
            return;
        }

        state.open_disputes.remove(&tx_id);
        tracing::info!(processor_id = self.id, client_id, tx_id, "dispute resolved");
    }

    fn process_chargeback(&mut self, client_id: ClientId, tx_id: TxId) {
        let state = self.account_storage.get_state_by_id_mut(client_id);

        let Some(&amount) = state.open_disputes.get(&tx_id) else {
            tracing::debug!(tx_id, client_id, "no open dispute found");
            return;
        };

        if let Err(err) = state.account.apply_chargeback(amount) {
            tracing::debug!(?err, client_id, tx_id, "ignoring chargeback");
            return;
        }

        state.open_disputes.remove(&tx_id);
        tracing::info!(
            processor_id = self.id,
            client_id,
            tx_id,
            "chargeback applied and account locked"
        );
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;

    use super::{Amount, Tx, TxProcessor};
    use crate::{
        core::types::BaseTx,
        storage::{account_storage::AccountStorage, in_memory_account::InMemoryAccountStorage},
    };

    fn amount(v: Decimal) -> Amount {
        Amount::from_dec(v)
    }

    fn base(client_id: u16, tx: u32) -> BaseTx {
        BaseTx { client_id, tx }
    }

    #[test]
    fn duplicate_deposit_is_ignored() {
        let mut processor = TxProcessor::<InMemoryAccountStorage>::new(0);

        let deposit = Tx::Deposit {
            base: base(1, 10),
            amount: amount(Decimal::new(15000, 4)),
        };

        processor.process_tx(deposit);
        processor.process_tx(Tx::Deposit {
            base: base(1, 10),
            amount: amount(Decimal::new(99999, 4)),
        });

        let state = processor.account_storage.get_state_by_id_mut(1);
        assert_eq!(*state.account.available, 15_000);
        assert_eq!(state.deposits.len(), 1);
    }

    #[test]
    fn dispute_then_resolve_updates_account_and_dispute_state() {
        let mut processor = TxProcessor::<InMemoryAccountStorage>::new(0);

        processor.process_tx(Tx::Deposit {
            base: base(2, 20),
            amount: amount(Decimal::new(20000, 4)),
        });
        processor.process_tx(Tx::Dispute { base: base(2, 20) });

        {
            let state = processor.account_storage.get_state_by_id_mut(2);
            assert_eq!(*state.account.available, 0);
            assert_eq!(*state.account.held, 20_000);
            assert_eq!(state.open_disputes.len(), 1);
        }

        processor.process_tx(Tx::Resolve { base: base(2, 20) });

        let state = processor.account_storage.get_state_by_id_mut(2);
        assert_eq!(*state.account.available, 20_000);
        assert_eq!(*state.account.held, 0);
        assert!(state.open_disputes.is_empty());
    }

    #[test]
    fn chargeback_locks_account_and_blocks_future_deposit() {
        let mut processor = TxProcessor::<InMemoryAccountStorage>::new(0);

        processor.process_tx(Tx::Deposit {
            base: base(3, 30),
            amount: amount(Decimal::new(30000, 4)),
        });
        processor.process_tx(Tx::Dispute { base: base(3, 30) });
        processor.process_tx(Tx::Chargeback { base: base(3, 30) });
        processor.process_tx(Tx::Deposit {
            base: base(3, 31),
            amount: amount(Decimal::new(10000, 4)),
        });

        let state = processor.account_storage.get_state_by_id_mut(3);
        assert!(state.account.locked);
        assert_eq!(*state.account.total, 0);
        assert_eq!(*state.account.available, 0);
        assert_eq!(state.deposits.len(), 1);
    }
}
