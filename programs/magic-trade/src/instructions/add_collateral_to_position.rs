use anchor_lang::prelude::*;
use crate::{
    constants::*, error::PlatformError, market::OraclePrice, state::{basket::Basket, custody::Custody, market::Market, pool::Pool},
    COLLATERAL_PRICE_MAX_AGE,
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
    #[account(address = pool.collateral_oracle)]
    pub collateral_oracle: UncheckedAccount<'info>,

    /// CHECK: Oracle account for lock custody price
    #[account(address = lock_custody.oracle)]
    pub lock_oracle: UncheckedAccount<'info>,
}

#[event]
pub struct AddCollateralLog {
    pub owner: Pubkey,
    pub market: Pubkey,
    pub amount: u64,
    pub amount_usd: u64,
    pub final_collateral_usd: u64,
    pub locked_amount: u64,
    pub custody_owned: u64,
    pub custody_reserved: u64,
}

pub fn handler(ctx: Context<AddCollateralToPosition>, amount: u64) -> Result<()> {
    msg!("Validate inputs");
    require!(amount > 0, PlatformError::InvalidInput);

    let basket = &mut ctx.accounts.basket;
    let collateral_custody = &mut ctx.accounts.collateral_custody;
    let lock_custody = &mut ctx.accounts.lock_custody;
    let market_key = ctx.accounts.market.key();

    msg!("Get position index");
    let position_index = basket
        .get_position_index(&market_key)
        .ok_or(PlatformError::PositionNotFound)?;

    let position = &mut basket.positions[position_index].position;

    msg!("Fetch collateral price from oracle");
    let curtime = Clock::get()?.unix_timestamp;
    let collateral_price = OraclePrice::from_pyth(
        &ctx.accounts.collateral_oracle,
        curtime,
        COLLATERAL_PRICE_MAX_AGE,
    )?;

    msg!("Fetch lock custody price from oracle");
    let lock_price = OraclePrice::from_pyth(
        &ctx.accounts.lock_oracle,
        curtime,
        lock_custody.max_price_age as i64,
    )?;

    msg!("Convert collateral amount to USD");
    let amount_usd = collateral_price.get_asset_amount_usd(amount, collateral_custody.decimals)?;

    msg!("Calculate locked amount in lock custody tokens");
    let locked_amount = lock_price.get_token_amount(amount_usd, lock_custody.decimals)?;

    msg!("Calculate final collateral after addition");
    let final_collateral_usd = position.collateral_usd
        .checked_add(amount_usd)
        .ok_or(PlatformError::MathError)?;

    msg!("Check minimum collateral requirement");
    require!(
        final_collateral_usd >= collateral_custody.margin_params.min_collateral_usd as u64,
        PlatformError::MinCollateral
    );

    msg!("Transfer collateral from reserved to owned");
    collateral_custody.reserved_to_owned(amount)?;

    msg!("Lock funds in lock custody");
    lock_custody.lock_funds(locked_amount)?;

    msg!("Update position collateral USD");
    position.collateral_usd = final_collateral_usd;

    msg!("Update position locked amount");
    position.locked_amount = position.locked_amount
        .checked_add(locked_amount)
        .ok_or(PlatformError::MathError)?;

    msg!("Emit event");
    emit!(AddCollateralLog {
        owner: ctx.accounts.owner.key(),
        market: market_key,
        amount,
        amount_usd,
        final_collateral_usd,
        locked_amount,
        custody_owned: collateral_custody.assets.owned,
        custody_reserved: collateral_custody.assets.reserved,
    });

    Ok(())
}