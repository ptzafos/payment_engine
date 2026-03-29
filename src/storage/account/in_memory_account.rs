use crate::{
    core::types::{Account, ClientId},
    storage::account::account_storage::AccountStorage,
};

#[derive(Default)]
pub struct InMemoryAccountStorage {
    pub storage: hashbrown::HashMap<ClientId, Account>,
}

impl AccountStorage for InMemoryAccountStorage {
    fn save(&mut self, account: Account) {
        self.storage.insert(account.client_id, account);
    }

    fn get_account_by_id(&mut self, client_id: &ClientId) -> Account {
        self.storage
            .entry(*client_id)
            .or_insert(Account {
                client_id: *client_id,
                ..<_>::default()
            })
            .clone()
    }

    fn report_state(&self) -> Vec<Account> {
        self.storage.values().cloned().collect::<Vec<_>>()
    }
}
