use anchor_lang::prelude::*;
use crate::{
    COLLATERAL_PRICE_MAX_AGE, constants::*, error::PlatformError, market::OraclePrice, math, state::{basket::Basket, custody::Custody, market::Market, pool::Pool}
};

#[derive(Accounts)]
pub struct AddCollateralToPosition<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [BASKET_SEED, owner.key().as_ref()],
        bump = basket.basket_bump
    )]
    pub basket: Account<'info, Basket>,

    #[account(
        seeds = [
            MARKET_SEED,
            market.target_custody.key().as_ref(),
            market.lock_custody.key().as_ref(),
            &[market.side as u8]
        ],
        bump
    )]
    pub market: Account<'info, Market>,

    #[account(
        seeds = [POOL_SEED, &[pool.id]],
        bump = pool.pool_bump
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        seeds = [CUSTODY_SEED, pool.key().as_ref(), &[target_custody.id]],
        bump = target_custody.custody_bump
    )]
    pub target_custody: Account<'info, Custody>,

    #[account(
        mut,
        seeds = [
            CUSTODY_SEED, 
            pool.key().as_ref(), 
            &[collateral_custody.id]
        ],
        bump = collateral_custody.custody_bump
    )]
    pub collateral_custody: Account<'info, Custody>,

    #[account(
        mut,
        seeds = [
            CUSTODY_SEED, 
            pool.key().as_ref(), 
            &[lock_custody.id]
        ],
        bump = lock_custody.custody_bump
    )]
    pub lock_custody: Account<'info, Custody>,

    /// CHECK: Oracle account validated by address
    #[account(address = target_custody.oracle)]
    pub target_oracle: UncheckedAccount<'info>,

    /// CHECK: Oracle account validated by address
    #[account(address = pool.collateral_oracle)]
    pub collateral_oracle: UncheckedAccount<'info>,

    /// CHECK: Oracle account for lock custody price
    #[account(address = lock_custody.oracle)]
    pub lock_oracle: UncheckedAccount<'info>,
}

#[event]
pub struct AddCollateralToPositionLog {
    pub owner: Pubkey,
    pub market: Pubkey,
    pub collateral_amount: u64,      // ← Better name than "amount"
    pub collateral_usd: u64,          // ← You have this
    pub size_amount: u64,             // ← Missing! Important to track size increase
    pub size_usd: u64,                // ← Missing! Important to track position growth
    pub locked_amount: u64,           // ← Good
    pub position_leverage: u128,      // ← Could add leverage for monitoring
    pub custody_owned: u64,           // ← Good
    pub custody_reserved: u64,        // ← Good
    pub timestamp: i64,               // ← Good for audit trail
}

pub fn handler(ctx: Context<AddCollateralToPosition>, collateral_amount: u64, size_amount: u64) -> Result<()> {

    let basket = &mut ctx.accounts.basket;
    let market_key = ctx.accounts.market.key();

    require!(
        basket.get_deposit_amount(&ctx.accounts.pool.key()) >= collateral_amount,
        PlatformError::InvalidBasketState
    );

    let current_time = Clock::get()?.unix_timestamp;
    let entry_price = OraclePrice::from_pyth(
        &ctx.accounts.target_oracle,
        current_time,
        ctx.accounts.target_custody.max_price_age as i64,
    )?;

    let lock_price = OraclePrice::from_pyth(
        &ctx.accounts.lock_oracle,
        current_time,
        ctx.accounts.lock_custody.max_price_age as i64,
    )?;
    
    let collateral_price = OraclePrice::from_pyth(
        &ctx.accounts.collateral_oracle,
        current_time,
        COLLATERAL_PRICE_MAX_AGE,
    )?;

    let size_usd = entry_price.get_asset_amount_usd(size_amount, ctx.accounts.target_custody.decimals)?;

    let entry_fee_usd = ctx
        .accounts
        .pool
        .get_fee_value(ctx.accounts.target_custody.trade_fee, size_usd)?;

    let collateral_usd = math::checked_sub(
        collateral_price.get_asset_amount_usd(collateral_amount, COLLATERAL_DECIMALS)?,
        entry_fee_usd,
    )?;

    let lock_amount = lock_price.get_token_amount(
        size_usd.saturating_add(collateral_usd),
        ctx.accounts.lock_custody.decimals,
    )?;

    let position_index = basket
        .get_position_index(&market_key)
        .ok_or(PlatformError::PositionNotFound)?;

    let position = &mut basket.positions[position_index].position;
    require_eq!(position.is_open(), true, PlatformError::InvalidBasketState);

    if ctx.accounts.collateral_custody.key() == ctx.accounts.lock_custody.key() {
        ctx.accounts.lock_custody.reserved_to_owned(collateral_amount)?;
        ctx.accounts.lock_custody.lock_funds(lock_amount)?;
    } else {
        ctx.accounts.collateral_custody.reserved_to_owned(collateral_amount)?;
        ctx.accounts.lock_custody.lock_funds(lock_amount)?;
    }

    position.size_amount = position.size_amount.checked_add(size_amount).ok_or(PlatformError::MathError)?;
    position.size_usd = position.size_usd.checked_add(size_usd).ok_or(PlatformError::MathError)?;
    position.locked_amount = position.locked_amount.checked_add(lock_amount).ok_or(PlatformError::MathError)?;
    position.collateral_usd = position.collateral_usd.checked_add(collateral_usd).ok_or(PlatformError::MathError)?;

    let leverage = position.get_leverage_and_margin(
        ctx.accounts.market.side,
        &entry_price,
        current_time,
        ctx.accounts.target_custody.margin_params.virtual_delay,
        true,
        entry_fee_usd
    )?.0;

    require_gte!(
        ctx.accounts.target_custody.margin_params.max_init_leverage as u128,
        leverage,
        PlatformError::MaxInitLeverage
    );

    basket.process_withdrawal(ctx.accounts.pool.key(), collateral_amount);    

    emit!(AddCollateralToPositionLog {
        owner: ctx.accounts.owner.key(),
        market: market_key,
        collateral_amount,
        collateral_usd,
        size_amount,
        size_usd,
        locked_amount: ctx.accounts.basket.positions[position_index].position.locked_amount,
        position_leverage: leverage,
        custody_owned: ctx.accounts.collateral_custody.assets.owned,
        custody_reserved: ctx.accounts.collateral_custody.assets.reserved,
        timestamp: current_time,
    });
    Ok(())
}