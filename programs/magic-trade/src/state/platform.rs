use anchor_lang::prelude::*;

use crate::{math, RATE_POWER};
#[account]
#[derive(InitSpace, Debug)]
pub struct Platform {
    pub version: u8,
    pub platform_bump: u8,
    pub token_authority_bump: u8,
    pub lamport_bank_bump: u8,
    pub pool_count: u8,
    pub padding: [u8; 3],
    pub permissions: Permissions,
    pub admin: Pubkey,
}

#[derive(Copy, Clone, AnchorSerialize, AnchorDeserialize, Default, Debug, InitSpace)]
pub struct Permissions {
    pub liquidity_add: bool,
    pub liquidity_remove: bool,
    pub trade_init: bool,
    pub trade_maint: bool,
    pub trade_liquidation: bool,
    pub padding: [u8; 3],
}
