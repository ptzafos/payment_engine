use crate::core::types::{Account, ClientId};

pub trait AccountStorage: Default + Send {
    fn save(&mut self, account: Account);
    fn get_account_by_id(&mut self, client_id: &ClientId) -> Account;
    fn report_state(&self) -> Vec<Account>;
}
