use crate::core::types::{Account, ClientId};

pub trait AccountStorage: Default + Send {
    fn save(&mut self, account: Account);
    fn retrieve(&mut self, client_id: ClientId) -> Account;
    // fn report_state() -> Vec<Account>
}
