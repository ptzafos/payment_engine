use crate::{
    collections::Map,
    core::types::{Account, AccountState, ClientId},
    storage::account_storage::AccountStorage,
};

#[derive(Default)]
pub struct InMemoryAccountStorage {
    pub storage: Map<ClientId, AccountState>,
}

impl AccountStorage for InMemoryAccountStorage {
    fn get_state_by_id_mut(&mut self, client_id: ClientId) -> &mut AccountState {
        self.storage
            .entry(client_id)
            .or_insert_with(|| AccountState {
                account: Account {
                    client_id,
                    ..Account::default()
                },
                ..AccountState::default()
            })
    }

    fn report_state(&self) -> Vec<Account> {
        self.storage
            .values()
            .map(|state| state.account.clone())
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
    use super::InMemoryAccountStorage;
    use crate::storage::account_storage::AccountStorage;

    #[test]
    fn creates_account_state_on_first_access() {
        let mut storage = InMemoryAccountStorage::default();

        {
            let state = storage.get_state_by_id_mut(42);
            assert_eq!(state.account.client_id, 42);
            assert!(state.deposits.is_empty());
            assert!(state.open_disputes.is_empty());
        }

        assert_eq!(storage.storage.len(), 1);
    }

    #[test]
    fn report_state_returns_all_accounts() {
        let mut storage = InMemoryAccountStorage::default();

        storage.get_state_by_id_mut(1).account.locked = true;
        storage.get_state_by_id_mut(2).account.locked = false;

        let mut accounts = storage.report_state();
        accounts.sort_by_key(|account| account.client_id);

        assert_eq!(accounts.len(), 2);
        assert_eq!(accounts[0].client_id, 1);
        assert!(accounts[0].locked);
        assert_eq!(accounts[1].client_id, 2);
        assert!(!accounts[1].locked);
    }
}
