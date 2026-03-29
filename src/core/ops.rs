use crate::core::types::{Account, Amount};

pub trait AccountOps {
    fn apply_deposit(&mut self, amount: Amount);
    fn try_apply_withdrawal(&mut self, amount: Amount) -> bool;
    fn apply_dispute(&mut self, disputed_amount: Amount);
    fn apply_resolve(&mut self, disputed_amount: Amount);
    fn apply_chargeback(&mut self, disputed_amount: Amount);
}

impl AccountOps for Account {
    fn apply_deposit(&mut self, amount: Amount) {
        if self.locked {
            return;
        }

        self.available += amount;
        self.total += amount;
    }

    fn try_apply_withdrawal(&mut self, amount: Amount) -> bool {
        if self.locked {
            return false;
        }

        if amount > self.available {
            return false;
        }

        self.available -= amount;
        self.total -= amount;
        true
    }

    fn apply_dispute(&mut self, disputed_amount: Amount) {
        if self.locked {
            return;
        }

        self.available -= disputed_amount;
        self.held += disputed_amount;
    }

    fn apply_resolve(&mut self, disputed_amount: Amount) {
        if self.locked {
            return;
        }

        self.held -= disputed_amount;
        self.available += disputed_amount;
    }

    fn apply_chargeback(&mut self, disputed_amount: Amount) {
        if self.locked {
            return;
        }

        self.held -= disputed_amount;
        self.total -= disputed_amount;
        self.locked = true;
    }
}
