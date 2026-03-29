use crate::core::types::{Account, AccountState, ClientId};

pub trait AccountStorage: Default + Send {
    fn get_state_by_id_mut(&mut self, client_id: ClientId) -> &mut AccountState;
    fn report_state(&self) -> Vec<Account>;
}
