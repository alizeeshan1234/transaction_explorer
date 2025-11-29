use anchor_lang::prelude::*;

use crate::{market::Position, MAX_MARKETS, MAX_POOLS};

#[account]
#[derive(Default, Debug, InitSpace)]
pub struct Basket {
    pub basket_bump: u8,
    pub padding: [u8; 7],
    #[max_len(MAX_POOLS)]
    pub deposits: Vec<Ledger>,
    #[max_len(MAX_MARKETS)]
    pub positions: Vec<PositionMeta>,
}

#[derive(Clone, Default, Debug, AnchorDeserialize, AnchorSerialize, InitSpace)]
pub struct PositionMeta {
    pub market: Pubkey,
    pub position: Position,
}

#[derive(Clone, Default, Debug, AnchorDeserialize, AnchorSerialize, InitSpace)]
pub struct Ledger {
    pub pool: Pubkey,
    pub amount: u64,
}

impl Basket {
    pub fn get_position_index(&self, market_key: &Pubkey) -> Option<usize> {
        self.positions
            .iter()
            .position(|pos_meta| &pos_meta.market == market_key)
    }
    pub fn get_deposit_index(&self, pool_key: &Pubkey) -> Option<usize> {
        self.deposits
            .iter()
            .position(|ledger| &ledger.pool == pool_key)
    }
    pub fn get_deposit_amount(&self, pool_key: &Pubkey) -> u64 {
        self.get_deposit_index(pool_key)
            .map_or(0, |index| self.deposits[index].amount)
    }
    pub fn process_deposit(&mut self, pool_key: Pubkey, amount: u64) {
        if let Some(index) = self.get_deposit_index(&pool_key) {
            self.deposits[index].amount = self.deposits[index].amount.saturating_add(amount);
        } else {
            self.deposits.push(Ledger {
                pool: pool_key,
                amount,
            });
        }
    }
    pub fn process_withdrawal(&mut self, pool_key: Pubkey, amount: u64) {
        if let Some(index) = self.get_deposit_index(&pool_key) {
            self.deposits[index].amount = self.deposits[index].amount.saturating_sub(amount);
        }
    }
}
