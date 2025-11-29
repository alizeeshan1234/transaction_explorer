use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};

use crate::{
    constants::{CUSTODY_SEED, MAX_CUSTODIES, PLATFORM_SEED, POOL_SEED},
    custody::{Assets, Custody, MarginParams},
    error::PlatformError,
    platform::{Permissions, Platform},
    pool::Pool,
    TOKEN_ACCOUNT_SEED, TOKEN_AUTHORITY_SEED,
};

#[derive(Accounts)]
#[instruction(custody_id: u8)]
pub struct InitializeCustody<'info> {
    #[account(
        mut,
        seeds = [PLATFORM_SEED],
        bump = platform.platform_bump,
        has_one = admin
    )]
    pub platform: Account<'info, Platform>,
    /// CHECK: Token mint authority PDA
    #[account(
        seeds = [TOKEN_AUTHORITY_SEED],
        bump = platform.token_authority_bump
    )]
    pub transfer_authority: AccountInfo<'info>,
    #[account(
        mut,
        seeds = [POOL_SEED, &[pool.id]],
        bump = pool.pool_bump
    )]
    pub pool: Account<'info, Pool>,
    #[account(
        init,
        payer = admin,
        space = 8 + Custody::INIT_SPACE,
        seeds = [CUSTODY_SEED, pool.key().as_ref(), &[custody_id]],
        bump
    )]
    pub custody: Account<'info, Custody>,
    pub token_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = admin,
        token::mint = token_mint,
        token::authority = transfer_authority,
        seeds = [TOKEN_ACCOUNT_SEED, custody.key().as_ref()],
        bump
    )]
    pub token_account: Account<'info, TokenAccount>,
    /// CHECK: Oracle account is recorded for reference and validated off-chain
    pub oracle: UncheckedAccount<'info>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, anchor_spl::token::Token>,
}

#[allow(clippy::too_many_arguments)]
pub fn handler(
    ctx: Context<InitializeCustody>,
    custody_id: u8,
    decimals: u8,
    stablecoin: bool,
    is_virtual: bool,
    permissions: Permissions,
    max_price_age: u64,
    margin_params: MarginParams,
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    require!(
        pool.custodies.len() < MAX_CUSTODIES as usize,
        PlatformError::InvalidPoolState
    );

    let custody_key = ctx.accounts.custody.key();
    require!(
        !pool.custodies.contains(&custody_key),
        PlatformError::InvalidCustodyState
    );

    pool.custodies.push(custody_key);
    pool.custody_count = pool.custodies.len() as u8;

    let custody = &mut ctx.accounts.custody;
    custody.id = custody_id;
    custody.custody_bump = ctx.bumps.custody;
    custody.decimals = decimals;
    custody.stablecoin = stablecoin;
    custody.is_virtual = is_virtual;
    custody.permissions = permissions;
    custody.token_mint = ctx.accounts.token_mint.key();
    custody.token_account = ctx.accounts.token_account.key();
    custody.oracle = ctx.accounts.oracle.key();
    custody.max_price_age = max_price_age;
    custody.margin_params = margin_params;
    custody.assets = Assets::default();

    Ok(())
}
