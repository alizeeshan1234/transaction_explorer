use anchor_lang::prelude::*;
use crate::{
    constants::*, error::PlatformError, state::{basket::Basket, custody::Custody, market::Market, pool::Pool}
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
        mut,
        seeds = [
            CUSTODY_SEED, 
            pool.key().as_ref(), 
            &[collateral_custody.id]
        ],
        bump = collateral_custody.custody_bump
    )]
    pub collateral_custody: Account<'info, Custody>,
}

pub fn handler(ctx: Context<AddCollateralToPosition>, amount: u64) -> Result<()> {

    let basket = &mut ctx.accounts.basket;
    let custody = &mut ctx.accounts.collateral_custody;
    let market_key = ctx.accounts.market.key();

    let position_index = basket
        .get_position_index(&market_key)
        .ok_or(PlatformError::PositionNotFound)?;

    // Decrement reserved assets
    custody.assets.reserved = custody
        .assets
        .reserved
        .checked_sub(amount)
        .ok_or(PlatformError::InsufficientCollateral)?;

    // Increment owned assets
    custody.assets.owned = custody
        .assets
        .owned
        .checked_add(amount)
        .ok_or(PlatformError::MathError)?;

    // Add to position
    basket.positions[position_index].position.collateral_usd = 
        basket.positions[position_index]
            .position
            .collateral_usd
            .checked_add(amount)
            .ok_or(PlatformError::MathError)?;

     emit!(CollateralAdded {
        owner: ctx.accounts.owner.key(),
        market: market_key,
        amount,
        new_collateral: basket.positions[position_index].position.collateral_usd,
        custody_owned: custody.assets.owned,
        custody_reserved: custody.assets.reserved,
    });

    Ok(())
}

#[event]
pub struct CollateralAdded {
    pub owner: Pubkey,
    pub market: Pubkey,
    pub amount: u64,
    pub new_collateral: u64,
    pub custody_owned: u64,
    pub custody_reserved: u64,
}