use crate::core::types::{Account, Amount};

impl Account {
    pub fn apply_deposit(&mut self, amount: Amount) -> Result<(), AccountError> {
        if self.locked {
            return Err(AccountError::AccountLocked);
        }

        self.available += amount;
        self.total += amount;
        Ok(())
    }

    pub fn try_apply_withdrawal(&mut self, amount: Amount) -> Result<(), AccountError> {
        if self.locked {
            return Err(AccountError::AccountLocked);
        }

        if amount > self.available {
            return Err(AccountError::InsufficientAvailableFunds {
                requested: amount,
                available: self.available,
            });
        }

        self.available -= amount;
        self.total -= amount;
        Ok(())
    }

    pub fn apply_dispute(&mut self, disputed_amount: Amount) -> Result<(), AccountError> {
        if self.locked {
            return Err(AccountError::AccountLocked);
        }

        self.available -= disputed_amount;
        self.held += disputed_amount;
        Ok(())
    }

    pub fn apply_resolve(&mut self, disputed_amount: Amount) -> Result<(), AccountError> {
        if self.locked {
            return Err(AccountError::AccountLocked);
        }

        if disputed_amount > self.held {
            return Err(AccountError::InsufficientHeldFunds {
                requested: disputed_amount,
                held: self.held,
            });
        }

        self.held -= disputed_amount;
        self.available += disputed_amount;
        Ok(())
    }

    pub fn apply_chargeback(&mut self, disputed_amount: Amount) -> Result<(), AccountError> {
        if self.locked {
            return Err(AccountError::AccountLocked);
        }

        if disputed_amount > self.held {
            return Err(AccountError::InsufficientHeldFunds {
                requested: disputed_amount,
                held: self.held,
            });
        }

        self.held -= disputed_amount;
        self.total -= disputed_amount;
        self.locked = true;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AccountError {
    #[error("account is locked")]
    AccountLocked,
    #[error("insufficient available funds: requested {requested:?}, available {available:?}")]
    InsufficientAvailableFunds {
        requested: Amount,
        available: Amount,
    },
    #[error("insufficient held funds: requested {requested:?}, held {held:?}")]
    InsufficientHeldFunds { requested: Amount, held: Amount },
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;

    use crate::core::types::{Account, Amount};

    fn amount(v: Decimal) -> Amount {
        Amount::from_dec(v)
    }

    fn account(client_id: u16) -> Account {
        Account {
            client_id,
            ..Account::default()
        }
    }

    #[test]
    fn deposit_and_withdrawal_update_balances() {
        let mut acc = account(1);

        acc.apply_deposit(amount(Decimal::new(25, 1)))
            .expect("deposit should succeed");
        acc.try_apply_withdrawal(amount(Decimal::new(10, 1)))
            .expect("withdrawal should succeed");

        assert_eq!(acc.available, amount(Decimal::new(15, 1)));
        assert_eq!(acc.held, amount(Decimal::ZERO));
        assert_eq!(acc.total, amount(Decimal::new(15, 1)));
        assert!(!acc.locked);
    }

    #[test]
    fn withdrawal_fails_when_funds_are_insufficient() {
        let mut acc = account(1);

        acc.apply_deposit(amount(Decimal::new(10, 1)))
            .expect("deposit should succeed");

        let result = acc.try_apply_withdrawal(amount(Decimal::new(15, 1)));

        assert!(matches!(
            result,
            Err(super::AccountError::InsufficientAvailableFunds { .. })
        ));
        assert_eq!(acc.available, amount(Decimal::new(10, 1)));
        assert_eq!(acc.total, amount(Decimal::new(10, 1)));
    }

    #[test]
    fn dispute_then_resolve_moves_funds_between_available_and_held() {
        let mut acc = account(1);

        acc.apply_deposit(amount(Decimal::new(20, 1)))
            .expect("deposit should succeed");
        acc.apply_dispute(amount(Decimal::new(5, 1)))
            .expect("dispute should succeed");

        assert_eq!(acc.available, amount(Decimal::new(15, 1)));
        assert_eq!(acc.held, amount(Decimal::new(5, 1)));
        assert_eq!(acc.total, amount(Decimal::new(20, 1)));

        acc.apply_resolve(amount(Decimal::new(5, 1)))
            .expect("resolve should succeed");

        assert_eq!(acc.available, amount(Decimal::new(20, 1)));
        assert_eq!(acc.held, amount(Decimal::ZERO));
        assert_eq!(acc.total, amount(Decimal::new(20, 1)));
    }

    #[test]
    fn chargeback_locks_account_and_blocks_future_operations() {
        let mut acc = account(1);

        acc.apply_deposit(amount(Decimal::new(20, 1)))
            .expect("deposit should succeed");
        acc.apply_dispute(amount(Decimal::new(20, 1)))
            .expect("dispute should succeed");
        acc.apply_chargeback(amount(Decimal::new(20, 1)))
            .expect("chargeback should succeed");

        assert!(acc.locked);
        assert_eq!(acc.available, amount(Decimal::ZERO));
        assert_eq!(acc.held, amount(Decimal::ZERO));
        assert_eq!(acc.total, amount(Decimal::ZERO));

        let result = acc.apply_deposit(amount(Decimal::new(10, 1)));
        assert!(matches!(result, Err(super::AccountError::AccountLocked)));
    }
}
