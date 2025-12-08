use anchor_lang::prelude::*;
use crate::{
    COLLATERAL_PRICE_MAX_AGE, constants::*, error::PlatformError, market::OraclePrice, math, state::{basket::Basket, custody::Custody, market::Market, pool::Pool}
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
        mut,
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
pub struct RemoveCollateralFromPositionLog {
    pub owner: Pubkey,
    pub market: Pubkey,
    pub collateral_amount: u64,
    pub collateral_usd: u64,
    pub size_amount: u64,
    pub size_usd: u64,
    pub unlocked_amount: u64,
    pub position_leverage: u128,
    pub custody_owned: u64,
    pub custody_reserved: u64,
    pub timestamp: i64,
}

pub fn handler(ctx: Context<RemoveCollateralFromPosition>, collateral_amount: u64, size_amount: u64) -> Result<()> {

    let basket = &mut ctx.accounts.basket;
    let market_key = ctx.accounts.market.key();

    msg!("Owner: {}", ctx.accounts.owner.key());
    msg!("Market: {}", market_key);
    msg!("Input collateral_amount: {}", collateral_amount);
    msg!("Input size_amount: {}", size_amount);

    let current_time = Clock::get()?.unix_timestamp;
    msg!("Current time: {}", current_time);

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

    msg!("Oracle Prices");
    msg!("Entry price: {} (exponent: {})", entry_price.price, entry_price.exponent);
    msg!("Lock price: {} (exponent: {})", lock_price.price, lock_price.exponent);
    msg!("Collateral price: {} (exponent: {})", collateral_price.price, collateral_price.exponent);

    let size_usd = entry_price.get_asset_amount_usd(size_amount, ctx.accounts.target_custody.decimals)?;

    let collateral_usd = collateral_price.get_asset_amount_usd(collateral_amount, COLLATERAL_DECIMALS)?;

    let unlock_amount = lock_price.get_token_amount(
        size_usd.saturating_add(collateral_usd),
        ctx.accounts.lock_custody.decimals,
    )?;

    msg!("Calculated Values");
    msg!("size_usd: {}", size_usd);
    msg!("collateral_usd: {}", collateral_usd);
    msg!("unlock_amount: {}", unlock_amount);

    let position_index = basket
        .get_position_index(&market_key)
        .ok_or(PlatformError::PositionNotFound)?;

    msg!("Position index found: {}", position_index);

    let position = &mut basket.positions[position_index].position;
    
    msg!("POSITION STATE BEFORE: ");
    msg!("Position is_open: {}", position.is_open());
    msg!("size_amount: {}", position.size_amount);
    msg!("size_usd: {}", position.size_usd);
    msg!("collateral_usd: {}", position.collateral_usd);
    msg!("locked_amount: {}", position.locked_amount);
    msg!("entry_price: {} (exp: {})", position.entry_price.price, position.entry_price.exponent);

    require_eq!(position.is_open(), true, PlatformError::InvalidBasketState);
    msg!("Position is open");

    require!(
        position.size_amount >= size_amount,
        PlatformError::InsufficientPositionSize
    );
    require!(
        position.collateral_usd >= collateral_usd,
        PlatformError::InsufficientCollateral
    );
    require!(
        position.locked_amount >= unlock_amount,
        PlatformError::InsufficientLockedAmount
    );

    msg!("CUSTODY STATE BEFORE: ");
    msg!("Collateral custody:");
    msg!("owned: {}", ctx.accounts.collateral_custody.assets.owned);
    msg!("reserved: {}", ctx.accounts.collateral_custody.assets.reserved);
    msg!("locked: {}", ctx.accounts.collateral_custody.assets.locked);
    
    msg!("Lock custody:");
    msg!("owned: {}", ctx.accounts.lock_custody.assets.owned);
    msg!("reserved: {}", ctx.accounts.lock_custody.assets.reserved);
    msg!("locked: {}", ctx.accounts.lock_custody.assets.locked);

    msg!("MARKET STATE BEFORE: ");
    msg!("Market collective_position:");
    msg!("size_amount: {}", ctx.accounts.market.collective_position.size_amount);
    msg!("size_usd: {}", ctx.accounts.market.collective_position.size_usd);
    msg!("collateral_usd: {}", ctx.accounts.market.collective_position.collateral_usd);
    msg!("locked_amount: {}", ctx.accounts.market.collective_position.locked_amount);
    msg!("open_positions: {}", ctx.accounts.market.open_positions);

    position.size_amount = position.size_amount.checked_sub(size_amount).ok_or(PlatformError::MathError)?;
    position.size_usd = position.size_usd.checked_sub(size_usd).ok_or(PlatformError::MathError)?;
    position.locked_amount = position.locked_amount.checked_sub(unlock_amount).ok_or(PlatformError::MathError)?;
    position.collateral_usd = position.collateral_usd.checked_sub(collateral_usd).ok_or(PlatformError::MathError)?;

    msg!("POSITION STATE AFTER UPDATE: ");
    msg!("size_amount: {}", position.size_amount);
    msg!("size_usd: {}", position.size_usd);
    msg!("collateral_usd: {}", position.collateral_usd);
    msg!("locked_amount: {}", position.locked_amount);

    if position.size_amount > 0 {
        let leverage = position.get_leverage_and_margin(
            ctx.accounts.market.side,
            &entry_price,
            current_time,
            ctx.accounts.target_custody.margin_params.virtual_delay,
            true,
            0
        )?.0;

        require_gte!(
            leverage,
            ctx.accounts.target_custody.margin_params.min_init_leverage as u128,
            PlatformError::MinInitLeverage
        );

        msg!("Position leverage after removal: {}", leverage);
    } else {
        msg!("Position fully closed");
    }

    if ctx.accounts.collateral_custody.key() == ctx.accounts.lock_custody.key() {
        msg!("Collateral custody == Lock custody (same account)");
        ctx.accounts.lock_custody.unlock_funds(unlock_amount)?;
        msg!("Unlocked {} funds", unlock_amount);
        ctx.accounts.lock_custody.owned_to_reserved(collateral_amount)?;
        msg!("Moved {} from owned to reserved", collateral_amount);
    } else {
        msg!("Collateral custody != Lock custody (different accounts)");
        ctx.accounts.lock_custody.unlock_funds(unlock_amount)?;
        msg!("Unlocked {} funds in lock_custody", unlock_amount);
        ctx.accounts.collateral_custody.owned_to_reserved(collateral_amount)?;
        msg!("Moved {} from owned to reserved in collateral_custody", collateral_amount);
    }

    msg!("CUSTODY STATE AFTER TRANSFERS: ");
    msg!("Collateral custody:");
    msg!("owned: {}", ctx.accounts.collateral_custody.assets.owned);
    msg!("reserved: {}", ctx.accounts.collateral_custody.assets.reserved);
    msg!("locked: {}", ctx.accounts.collateral_custody.assets.locked);
    
    msg!("Lock custody:");
    msg!("owned: {}", ctx.accounts.lock_custody.assets.owned);
    msg!("reserved: {}", ctx.accounts.lock_custody.assets.reserved);
    msg!("locked: {}", ctx.accounts.lock_custody.assets.locked);

    msg!("MARKET STATE BEFORE COLLECTIVE UPDATE: ");
    msg!("collective_position.size_amount: {} → {}", 
        ctx.accounts.market.collective_position.size_amount,
        ctx.accounts.market.collective_position.size_amount - size_amount
    );
    msg!("collective_position.size_usd: {} → {}", 
        ctx.accounts.market.collective_position.size_usd,
        ctx.accounts.market.collective_position.size_usd - size_usd
    );

    ctx.accounts.market.collective_position.size_amount = 
        math::checked_sub(ctx.accounts.market.collective_position.size_amount, size_amount)?;
    ctx.accounts.market.collective_position.size_usd = 
        math::checked_sub(ctx.accounts.market.collective_position.size_usd, size_usd)?;
    ctx.accounts.market.collective_position.locked_amount = 
        math::checked_sub(ctx.accounts.market.collective_position.locked_amount, unlock_amount)?;
    ctx.accounts.market.collective_position.collateral_usd = 
        math::checked_sub(ctx.accounts.market.collective_position.collateral_usd, collateral_usd)?;

    msg!("MARKET STATE AFTER COLLECTIVE UPDATE: ");
    msg!("collective_position.size_amount: {}", ctx.accounts.market.collective_position.size_amount);
    msg!("collective_position.size_usd: {}", ctx.accounts.market.collective_position.size_usd);
    msg!("collective_position.collateral_usd: {}", ctx.accounts.market.collective_position.collateral_usd);
    msg!("collective_position.locked_amount: {}", ctx.accounts.market.collective_position.locked_amount);

    let final_leverage = if position.size_amount > 0 {
        position.get_leverage_and_margin(
            ctx.accounts.market.side,
            &entry_price,
            current_time,
            ctx.accounts.target_custody.margin_params.virtual_delay,
            true,
            0
        )?.0
    } else {
        0
    };

    basket.process_deposit(ctx.accounts.pool.key(), collateral_amount);
    msg!("Processed deposit to basket");

    let final_deposit = basket.get_deposit_amount(&ctx.accounts.pool.key());
    msg!("Final deposit: {}", final_deposit);

    emit!(RemoveCollateralFromPositionLog {
        owner: ctx.accounts.owner.key(),
        market: market_key,
        collateral_amount,
        collateral_usd,
        size_amount,
        size_usd,
        unlocked_amount: unlock_amount,
        position_leverage: final_leverage,
        custody_owned: ctx.accounts.collateral_custody.assets.owned,
        custody_reserved: ctx.accounts.collateral_custody.assets.reserved,
        timestamp: current_time,
    });

    Ok(())
}