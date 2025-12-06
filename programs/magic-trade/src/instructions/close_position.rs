use anchor_lang::prelude::*;

use crate::{
    basket::Basket,
    constants::{BASKET_SEED, CUSTODY_SEED, MARKET_SEED, POOL_SEED},
    custody::Custody,
    error::PlatformError,
    market::{Market, OraclePrice},
    pool::Pool,
    COLLATERAL_DECIMALS, COLLATERAL_PRICE_MAX_AGE,
};

#[derive(Accounts)]
pub struct ClosePosition<'info> {
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

    #[account(
        mut,
        seeds = [CUSTODY_SEED, pool.key().as_ref(), &[collateral_custody.id]],
        bump = collateral_custody.custody_bump,
        address = pool.custodies[0], // Ensure collateral custody is the first custody
    )]
    pub collateral_custody: Account<'info, Custody>,

    /// CHECK: Oracle account validated by address
    #[account(address = target_custody.oracle)]
    pub target_oracle: UncheckedAccount<'info>,

    /// CHECK: Oracle account validated by address
    #[account(address = pool.collateral_oracle)]
    pub collateral_oracle: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<ClosePosition>) -> Result<()> {
    let basket = &mut ctx.accounts.basket;
    let pos_index = basket
        .positions
        .iter()
        .position(|p| p.market == ctx.accounts.market.key())
        .ok_or(PlatformError::InvalidPositionState)?;

    let position = basket.positions[pos_index].position;
    require!(position.is_open(), PlatformError::InvalidPositionState);

    let curtime = Clock::get()?.unix_timestamp;
    let exit_price = OraclePrice::from_pyth(
        &ctx.accounts.target_oracle,
        curtime,
        ctx.accounts.target_custody.max_price_age as i64,
    )?;
    let (leverage, margin_usd) = position.get_leverage_and_margin(
        ctx.accounts.market.side,
        &exit_price,
        curtime,
        ctx.accounts.lock_custody.margin_params.virtual_delay,
        false,
        ctx.accounts
            .pool
            .get_fee_value(ctx.accounts.target_custody.trade_fee, position.size_usd)?,
    )?;

    require_gte!(
        ctx.accounts.target_custody.margin_params.max_leverage as u128,
        leverage,
        PlatformError::MaxLeverage
    );

    let collateral_price = OraclePrice::from_pyth(
        &ctx.accounts.collateral_oracle,
        curtime,
        COLLATERAL_PRICE_MAX_AGE,
    )?;
    // convert back to token amount using target custody price
    let settle_amount = collateral_price.get_token_amount(margin_usd, COLLATERAL_DECIMALS)?;

    basket.process_deposit(ctx.accounts.pool.key(), settle_amount);
    basket.positions.swap_remove(pos_index);

    ctx.accounts.market.remove_position(&position)?;
    ctx.accounts
        .lock_custody
        .unlock_funds(position.locked_amount)?;
    if ctx.accounts.collateral_custody.key() == ctx.accounts.lock_custody.key() {
        ctx.accounts.lock_custody.owned_to_reserved(settle_amount)?;
        ctx.accounts.collateral_custody = ctx.accounts.lock_custody.clone();
    } else {
        ctx.accounts
            .collateral_custody
            .owned_to_reserved(settle_amount)?;
    }

    msg!("Position closed successfully!");
    Ok(())
}
