use anchor_lang::prelude::*;

use crate::{
    basket::Basket,
    constants::{BASKET_SEED, BPS_POWER, CUSTODY_SEED, MARKET_SEED, POOL_SEED},
    custody::Custody,
    error::PlatformError,
    market::{Market, OraclePrice},
    math,
    pool::Pool,
};

#[derive(Accounts)]
pub struct LiquidatePosition<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [BASKET_SEED, owner.key().as_ref()],
        bump = basket.basket_bump
    )]
    pub basket: Account<'info, Basket>,

    #[account(
        mut,
        seeds = [POOL_SEED, &[pool.id]],
        bump = pool.pool_bump
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        seeds = [
            MARKET_SEED,
            target_custody.key().as_ref(),
            lock_custody.key().as_ref(),
            &[market.side as u8]
        ],
        bump = market.market_bump
    )]
    pub market: Account<'info, Market>,

    #[account(
        seeds = [CUSTODY_SEED, pool.key().as_ref(), &[target_custody.id]],
        bump = target_custody.custody_bump
    )]
    pub target_custody: Account<'info, Custody>,

    #[account(
        seeds = [CUSTODY_SEED, pool.key().as_ref(), &[lock_custody.id]],
        bump = lock_custody.custody_bump
    )]
    pub lock_custody: Account<'info, Custody>,

    /// CHECK: Oracle account validated by address
    #[account(address = target_custody.oracle)]
    pub target_oracle: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<LiquidatePosition>) -> Result<()> {
    let basket = &mut ctx.accounts.basket;
    let pos_index = basket
        .positions
        .iter()
        .position(|p| p.market == ctx.accounts.market.key())
        .ok_or(PlatformError::InvalidPositionState)?;

    let position = basket.positions[pos_index].position;
    require!(position.is_open(), PlatformError::InvalidPositionState);

    // Compute current leverage in bps using target price
    let curtime = Clock::get()?.unix_timestamp;
    let price = OraclePrice::from_pyth(
        &ctx.accounts.target_oracle,
        curtime,
        ctx.accounts.target_custody.max_price_age as i64,
    )?;
    let (leverage, _margin_usd) = position.get_leverage_and_margin(
        ctx.accounts.market.side,
        &price,
        curtime,
        ctx.accounts.lock_custody.margin_params.virtual_delay,
        false,
        ctx.accounts
            .pool
            .get_fee_value(ctx.accounts.target_custody.trade_fee, position.size_usd)?,
    )?;

    require_gt!(
        leverage,
        ctx.accounts.lock_custody.margin_params.max_leverage as u128,
        PlatformError::MaxLeverage
    );

    // Drop position and update market aggregates
    basket.positions.swap_remove(pos_index);
    ctx.accounts.market.remove_position(&position)?;
    ctx.accounts
        .lock_custody
        .unlock_funds(position.locked_amount)?;

    Ok(())
}
