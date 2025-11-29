use anchor_lang::prelude::*;

use crate::{
    basket::{Basket, PositionMeta},
    constants::{BASKET_SEED, CUSTODY_SEED, MARKET_SEED, MAX_MARKETS, POOL_SEED},
    custody::Custody,
    error::PlatformError,
    market::{Market, OraclePrice, Position},
    math,
    pool::Pool,
    COLLATERAL_DECIMALS, COLLATERAL_PRICE_MAX_AGE,
};

#[derive(Accounts)]
pub struct OpenPosition<'info> {
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
        mut,
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
    #[account(address = lock_custody.oracle)]
    pub lock_oracle: UncheckedAccount<'info>,

    /// CHECK: Oracle account validated by address
    #[account(address = pool.collateral_oracle)]
    pub collateral_oracle: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<OpenPosition>, collateral_amount: u64, size_amount: u64) -> Result<()> {
    let basket = &mut ctx.accounts.basket;
    require!(
        basket.get_deposit_amount(&ctx.accounts.pool.key()) >= collateral_amount,
        PlatformError::InvalidBasketState
    );
    require!(
        basket.positions.len() < MAX_MARKETS as usize,
        PlatformError::InvalidBasketState
    );

    let curtime = Clock::get()?.unix_timestamp;
    let entry_price = OraclePrice::from_pyth(
        &ctx.accounts.target_oracle,
        curtime,
        ctx.accounts.target_custody.max_price_age as i64,
    )?;
    let lock_price = OraclePrice::from_pyth(
        &ctx.accounts.lock_oracle,
        curtime,
        ctx.accounts.lock_custody.max_price_age as i64,
    )?;
    let collateral_price = OraclePrice::from_pyth(
        &ctx.accounts.collateral_oracle,
        curtime,
        COLLATERAL_PRICE_MAX_AGE,
    )?;
    let size_usd =
        entry_price.get_asset_amount_usd(size_amount, ctx.accounts.target_custody.decimals)?;
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

    let position = Position {
        open_time: curtime,
        entry_price,
        size_amount: size_amount,
        size_usd,
        locked_amount: lock_amount,
        collateral_usd: collateral_usd,
        size_decimals: ctx.accounts.target_custody.decimals,
        locked_decimals: ctx.accounts.target_custody.decimals,
        padding: [0u8; 6],
    };

    require_gte!(
        ctx.accounts.target_custody.margin_params.max_init_leverage as u128,
        position
            .get_leverage_and_margin(
                ctx.accounts.market.side,
                &entry_price,
                curtime,
                ctx.accounts.target_custody.margin_params.virtual_delay,
                true,
                entry_fee_usd,
            )?
            .0,
        PlatformError::MaxInitLeverage
    );

    basket.positions.push(PositionMeta {
        market: ctx.accounts.market.key(),
        position,
    });
    basket.process_withdrawal(ctx.accounts.pool.key(), collateral_amount);

    ctx.accounts.market.add_position(&position)?;
    if ctx.accounts.collateral_custody.key() == ctx.accounts.lock_custody.key() {
        ctx.accounts
            .lock_custody
            .reserved_to_owned(collateral_amount)?;
        ctx.accounts.lock_custody.lock_funds(lock_amount)?;
        ctx.accounts.collateral_custody = ctx.accounts.lock_custody.clone();
    } else {
        ctx.accounts
            .collateral_custody
            .reserved_to_owned(collateral_amount)?;
        ctx.accounts.lock_custody.lock_funds(lock_amount)?;
    }
    Ok(())
}
