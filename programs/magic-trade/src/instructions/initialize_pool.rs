use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};

use crate::{
    constants::LP_DECIMALS,
    constants::{LP_MINT_SEED, PLATFORM_SEED, POOL_SEED},
    error::PlatformError,
    platform::Platform,
    pool::Pool,
    TOKEN_AUTHORITY_SEED,
};

#[derive(Accounts)]
#[instruction(pool_id: u8)]
pub struct InitializePool<'info> {
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
        init,
        payer = admin,
        mint::authority = transfer_authority,
        mint::freeze_authority = transfer_authority,
        mint::decimals = LP_DECIMALS,
        seeds = [LP_MINT_SEED, &[pool_id]],
        bump
    )]
    pub lp_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = admin,
        space = 8 + Pool::INIT_SPACE,
        seeds = [POOL_SEED, &[pool_id]],
        bump
    )]
    pub pool: Account<'info, Pool>,

    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(
    ctx: Context<InitializePool>,
    pool_id: u8,
    max_aum_usd: u64,
    buffer: u64,
    collateral_oracle: Pubkey,
) -> Result<()> {
    let platform = &mut ctx.accounts.platform;

    require!(
        pool_id == platform.pool_count,
        PlatformError::InvalidPoolState
    );
    platform.pool_count = platform
        .pool_count
        .checked_add(1)
        .ok_or(PlatformError::InvalidPoolState)?;

    let pool = &mut ctx.accounts.pool;
    pool.id = pool_id;
    pool.pool_bump = ctx.bumps.pool;
    pool.lp_mint_bump = ctx.bumps.lp_mint;
    pool.custody_count = 0;
    pool.max_aum_usd = max_aum_usd;
    pool.buffer = buffer;
    pool.raw_aum_usd = 0;
    pool.equity_usd = 0;
    pool.custodies = Vec::new();
    pool.markets = Vec::new();
    pool.collateral_oracle = collateral_oracle;
    Ok(())
}
