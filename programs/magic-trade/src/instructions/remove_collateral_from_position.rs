use anchor_lang::prelude::*;
use crate::{
    constants::*, error::PlatformError, market::OraclePrice, state::{basket::Basket, custody::Custody, market::Market, pool::Pool},
    COLLATERAL_PRICE_MAX_AGE, math,
};

#[derive(Accounts)]
pub struct RemoveCollateralFromPosition<'info> {
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
        mut,
        seeds = [
            CUSTODY_SEED, 
            pool.key().as_ref(), 
            &[collateral_custody.id]
        ],
        bump = collateral_custody.custody_bump
    )]
    pub collateral_custody: Account<'info, Custody>,

    /// CHECK: Oracle account validated by address
    #[account(address = pool.collateral_oracle)]
    pub collateral_oracle: UncheckedAccount<'info>,
}

#[event]
pub struct RemoveCollateralLog {
    pub owner: Pubkey,
    pub market: Pubkey,
    pub amount: u64,
    pub amount_usd: u64,
    pub remaining_collateral_usd: u64,
    pub custody_owned: u64,
    pub custody_reserved: u64,
}

pub fn handler(ctx: Context<RemoveCollateralFromPosition>, amount: u64) -> Result<()> {
    require!(amount > 0, PlatformError::InvalidInput);

    let basket = &mut ctx.accounts.basket;
    let custody = &mut ctx.accounts.collateral_custody;
    let market_key = ctx.accounts.market.key();

    let position_index = basket
        .get_position_index(&market_key)
        .ok_or(PlatformError::PositionNotFound)?;

    let position = &mut basket.positions[position_index].position;

    let curtime = Clock::get()?.unix_timestamp;
    let collateral_price = OraclePrice::from_pyth(
        &ctx.accounts.collateral_oracle,
        curtime,
        COLLATERAL_PRICE_MAX_AGE,
    )?;

    let amount_usd = collateral_price.get_asset_amount_usd(amount, custody.decimals)?;

    require!(
        position.collateral_usd >= amount_usd,
        PlatformError::InsufficientCollateral
    );

    require!(
        custody.assets.owned >= amount,
        PlatformError::InsufficientCollateral
    );

    position.collateral_usd = position.collateral_usd
        .checked_sub(amount_usd)
        .ok_or(PlatformError::MathError)?;

    custody.assets.owned = custody
        .assets
        .owned
        .checked_sub(amount)
        .ok_or(PlatformError::InsufficientCollateral)?;

    custody.assets.reserved = custody
        .assets
        .reserved
        .checked_add(amount)
        .ok_or(PlatformError::MathError)?;

    emit!(RemoveCollateralLog {
        owner: ctx.accounts.owner.key(),
        market: market_key,
        amount,
        amount_usd,
        remaining_collateral_usd: position.collateral_usd,
        custody_owned: custody.assets.owned,
        custody_reserved: custody.assets.reserved,
    });

    Ok(())
}