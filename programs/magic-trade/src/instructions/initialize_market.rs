use anchor_lang::prelude::*;

use crate::{
    constants::{CUSTODY_SEED, MARKET_SEED, MAX_MARKETS, PLATFORM_SEED, POOL_SEED},
    custody::Custody,
    error::PlatformError,
    market::{Market, Position, Side},
    platform::{Permissions, Platform},
    pool::Pool,
};

#[derive(Accounts)]
#[instruction(market_id: u8, side: Side)]
pub struct InitializeMarket<'info> {
    #[account(
        mut,
        seeds = [PLATFORM_SEED],
        bump = platform.platform_bump,
        has_one = admin
    )]
    pub platform: Account<'info, Platform>,
    #[account(
        mut,
        seeds = [POOL_SEED, &[pool.id]],
        bump = pool.pool_bump
    )]
    pub pool: Account<'info, Pool>,
    #[account(
        mut,
        seeds = [CUSTODY_SEED, pool.key().as_ref(), &[target_custody.id]],
        bump = target_custody.custody_bump
    )]
    pub target_custody: Account<'info, Custody>,
    #[account(
        mut,
        seeds = [CUSTODY_SEED, pool.key().as_ref(), &[lock_custody.id]],
        bump = lock_custody.custody_bump
    )]
    pub lock_custody: Account<'info, Custody>,
    #[account(
        init,
        payer = admin,
        space = 8 + Market::INIT_SPACE,
        seeds = [
            MARKET_SEED,
            target_custody.key().as_ref(),
            lock_custody.key().as_ref(),
            &[side as u8]
        ],
        bump
    )]
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[allow(clippy::too_many_arguments)]
pub fn handler(
    ctx: Context<InitializeMarket>,
    market_id: u8,
    side: Side,
    is_virtual: bool,
    permissions: Permissions,
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let target_custody = &mut ctx.accounts.target_custody;
    require!(
        pool.custodies.contains(&target_custody.key())
            && pool.custodies.contains(&ctx.accounts.lock_custody.key()),
        PlatformError::InvalidCustodyState
    );

    require!(
        pool.markets.len() < MAX_MARKETS as usize,
        PlatformError::InvalidMarketState
    );

    let market = &mut ctx.accounts.market;
    market.id = market_id;
    market.market_bump = ctx.bumps.market;
    market.side = side;
    market.is_virtual = is_virtual;
    market.permissions = permissions;
    market.target_custody = target_custody.key();
    market.lock_custody = ctx.accounts.lock_custody.key();
    market.open_positions = 0;
    market.collective_position = Position::default();

    pool.markets.push(market.key());
    target_custody.supported_markets.push(market.key());

    Ok(())
}
