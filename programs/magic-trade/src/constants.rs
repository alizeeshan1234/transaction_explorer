use anchor_lang::prelude::*;

#[constant]
pub const PLATFORM_SEED: &[u8] = b"platform";
#[constant]
pub const TOKEN_AUTHORITY_SEED: &[u8] = b"authority";
#[constant]
pub const LAMPORT_BANK_SEED: &[u8] = b"lamport_bank";
#[constant]
pub const POOL_SEED: &[u8] = b"pool";
#[constant]
pub const LP_MINT_SEED: &[u8] = b"lp_token";
#[constant]
pub const CUSTODY_SEED: &[u8] = b"custody";
#[constant]
pub const TOKEN_ACCOUNT_SEED: &[u8] = b"token_account";
#[constant]
pub const MARKET_SEED: &[u8] = b"market";
#[constant]
pub const BASKET_SEED: &[u8] = b"basket";
#[constant]
pub const MAX_POOLS: u8 = 10;
#[constant]
pub const MAX_CUSTODIES: u8 = 10;
#[constant]
pub const MAX_MARKETS: u8 = 20;
#[constant]
pub const BPS_DECIMALS: u8 = 4;
#[constant]
pub const BPS_POWER: u128 = 10u64.pow(BPS_DECIMALS as u32) as u128;
#[constant]
pub const USD_DECIMALS: u8 = 6;
#[constant]
pub const USD_POWER: u128 = 10u64.pow(USD_DECIMALS as u32) as u128;
#[constant]
pub const LP_DECIMALS: u8 = 6;
#[constant]
pub const LP_POWER: u128 = 10u64.pow(LP_DECIMALS as u32) as u128;
#[constant]
pub const RATE_DECIMALS: u8 = 9;
#[constant]
pub const RATE_POWER: u128 = 10u64.pow(RATE_DECIMALS as u32) as u128;
#[constant]
pub const COLLATERAL_DECIMALS: u8 = 6;
#[constant]
pub const COLLATERAL_PRICE_MAX_AGE: i64 = 600;
